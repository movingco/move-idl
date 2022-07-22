//! CLI for parsing an IDL from a Move package.
use std::path::PathBuf;

use anyhow::*;
use clitool::CliTool;
use move_idl::IDLBuilder;

/// Parses a Move workspace into a set of IDLs.
#[derive(clap::Parser)]
#[clap(name = "move-idl-parse", author, version)]
pub struct IDLParseTool {
    /// Path to the root of the Move workspace.
    #[clap(default_value = ".")]
    pub root: PathBuf,
    /// Output directory for the generated files.
    #[clap(short, long, default_value = "./build/idls/")]
    pub out_dir: PathBuf,

    /// Whether to generate module IDL JSON files for dependencies.
    #[clap(short, long)]
    pub with_dependencies: bool,
}

#[async_trait::async_trait]
impl CliTool<()> for IDLParseTool {
    async fn execute(self) -> Result<()> {
        let idl = IDLBuilder::load(&self.root)?.gen()?;

        std::fs::create_dir_all(&self.out_dir)?;

        let relevant_modules = if self.with_dependencies {
            let mut idl_mut = idl.clone();
            let mut modules = idl_mut.modules;
            modules.append(&mut idl_mut.dependencies);
            modules
        } else {
            idl.clone().modules
        };

        let modules_dir = &self.out_dir.join("modules");
        std::fs::create_dir_all(modules_dir)?;
        for (name, module_idl) in relevant_modules.iter() {
            let module_idl_path = &modules_dir
                .join(name.name().as_str())
                .with_extension("json");
            std::fs::write(module_idl_path, serde_json::to_string_pretty(&module_idl)?)?;
        }

        std::fs::write(
            &self.out_dir.join("combined.json"),
            serde_json::to_string_pretty(&idl)?,
        )?;

        Ok(())
    }
}
