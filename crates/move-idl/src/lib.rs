//! Move IDL generator.

use anyhow::*;
use errmapgen::{ErrmapGen, ErrmapOptions};
use generate::gen_module::{generate_idl_for_module, generate_idl_structs_for_module};
use move_core_types::account_address::AccountAddress;
pub use move_idl_types::*;
use move_model::model::GlobalEnv;
use move_package::{resolution::resolution_graph::ResolutionPackage, BuildConfig, ModelConfig};
use std::{collections::BTreeMap, path::Path};

pub mod convert;
pub mod generate;

fn normalize_doc_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub struct IDLBuilder {
    package: ResolutionPackage<AccountAddress>,
    env: GlobalEnv,
    /// Env where all files are targets. This allows us to get more doc strings.
    env_all_targets: GlobalEnv,
}

impl IDLBuilder {
    pub fn load(root_path: &Path) -> Result<IDLBuilder> {
        let build_config = BuildConfig {
            generate_docs: true,
            generate_abis: true,
            ..Default::default()
        };

        let resolution_graph = &build_config
            .clone()
            .resolution_graph_for_package(root_path)?;
        let root_package = &resolution_graph.root_package.package;

        let model = build_config.clone().move_model_for_package(
            root_path,
            ModelConfig {
                all_files_as_targets: false,
                target_filter: None,
            },
        )?;

        let env_all_targets = build_config.move_model_for_package(
            root_path,
            ModelConfig {
                all_files_as_targets: true,
                target_filter: None,
            },
        )?;

        Ok(IDLBuilder {
            package: resolution_graph.package_table[&root_package.name].clone(),
            env: model,
            env_all_targets,
        })
    }

    fn gen_aliases(&self) -> BTreeMap<String, AccountAddress> {
        self.package
            .resolution_table
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), *v))
            .collect()
    }

    /// Generates IDLs for all script modules in the environment (excluding the dependency set).
    pub fn gen(&self) -> Result<IDLPackage> {
        let aliases = self.gen_aliases();
        let mut modules: BTreeMap<String, IDLModule> = BTreeMap::new();

        let mut dependencies: BTreeMap<String, IDLModule> = BTreeMap::new();

        let error_mapping =
            ErrmapGen::new(&self.env_all_targets, &ErrmapOptions::default()).gen()?;

        for module in self.env.get_modules() {
            if !module.is_script_module() {
                let abi = generate_idl_for_module(&self.env, &error_mapping, module.clone())?;
                if module.is_target() {
                    modules.insert(abi.module_id.name().to_string(), abi);
                } else {
                    dependencies.insert(abi.module_id.name().to_string(), abi);
                }
            }
        }

        let structs = self
            .env
            .get_modules()
            .map(|module_env| generate_idl_structs_for_module(&self.env, module_env))
            .collect::<Result<Vec<Vec<IDLStruct>>>>()?
            .concat();

        Ok(IDLPackage {
            name: self.package.source_package.package.name.as_str().into(),
            modules,
            aliases,
            structs,
            errors: error_mapping,
            dependencies,
        })
    }
}
