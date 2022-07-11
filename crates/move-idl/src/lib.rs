//! Move IDL generator.

use anyhow::*;
use errmapgen::{ErrmapGen, ErrmapOptions};
use generate::gen_module::{generate_idl_for_module, generate_idl_structs_for_module};
use module_id::ModuleIdData;
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
pub use move_idl_types::*;
use move_model::model::GlobalEnv;
use move_package::{resolution::resolution_graph::ResolutionPackage, BuildConfig, ModelConfig};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

pub mod convert;
pub mod generate;
pub mod utils;

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
        Self::load_with_config(root_path, build_config)
    }

    pub fn load_with_config(root_path: &Path, build_config: BuildConfig) -> Result<IDLBuilder> {
        let resolution_graph = &build_config
            .clone()
            .resolution_graph_for_package(root_path)?;
        let root_package = &resolution_graph.root_package.package;

        let env_all_targets = build_config.clone().move_model_for_package(
            root_path,
            ModelConfig {
                all_files_as_targets: true,
                target_filter: None,
            },
        )?;

        let env = build_config.move_model_for_package(
            root_path,
            ModelConfig {
                all_files_as_targets: false,
                target_filter: None,
            },
        )?;

        Ok(IDLBuilder {
            package: resolution_graph.package_table[&root_package.name].clone(),
            env,
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

        let error_mapping =
            ErrmapGen::new(&self.env_all_targets, &ErrmapOptions::default()).gen()?;

        let structs = self
            .env_all_targets
            .get_modules()
            .map(|module_env| generate_idl_structs_for_module(&self.env_all_targets, module_env))
            .collect::<Result<Vec<Vec<IDLStruct>>>>()?
            .concat();

        let relevant_module_ids: HashSet<ModuleId> = self
            .env
            .get_modules()
            .map(|m| m.get_verified_module().self_id())
            .collect();

        let target_module_ids: HashSet<ModuleId> = self
            .env
            .get_target_modules()
            .iter()
            .map(|m| m.get_verified_module().self_id())
            .collect();

        let mut modules: BTreeMap<ModuleIdData, IDLModule> = BTreeMap::new();
        let mut dependencies: BTreeMap<ModuleIdData, IDLModule> = BTreeMap::new();
        for module in self.env_all_targets.get_modules() {
            let module_id = module.get_verified_module().self_id();
            if !relevant_module_ids.contains(&module_id) {
                continue;
            }
            if module.is_script_module() {
                continue;
            }

            let idl =
                generate_idl_for_module(&self.env_all_targets, &error_mapping, module.clone())?;
            if target_module_ids.contains(&module_id) {
                modules.insert(idl.module_id.clone(), idl);
            } else {
                dependencies.insert(idl.module_id.clone(), idl);
            }
        }

        Ok(IDLPackage {
            name: self.package.source_package.package.name.as_str().into(),
            modules,
            aliases,
            structs,
            errors: error_mapping.into(),
            dependencies,
        })
    }
}
