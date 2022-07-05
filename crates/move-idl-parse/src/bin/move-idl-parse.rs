use anyhow::*;
use clitool::CliTool;
use move_idl_parse::IDLParseTool;

#[tokio::main]
async fn main() -> Result<()> {
    IDLParseTool::execute_main().await
}
