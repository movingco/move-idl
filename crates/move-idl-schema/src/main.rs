//! CLI to generate a JSON schema for the Move IDL.

use anyhow::*;
use move_idl::IDLPackage;
use schemars::schema_for;

/// Generates the JSON schema for Move IDL.
#[derive(clap::Parser)]
#[clap(name = "move-idl-schema", author, version)]
pub struct IDLSchemaTool {}

fn main() -> Result<()> {
    let schema = schema_for!(IDLPackage);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
