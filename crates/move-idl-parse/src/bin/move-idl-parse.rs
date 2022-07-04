use anyhow::*;
use json_cli::CliTool;
use move_idl_parse::IDLParseTool;

#[tokio::main]
async fn main() -> Result<()> {
    IDLParseTool::execute_main().await
}
