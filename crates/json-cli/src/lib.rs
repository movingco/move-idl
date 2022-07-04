//! Helpers for creating CLIs with serializable JSON output.
//!
//! This package is largely based on code from Aptos CLI.

use std::process::exit;

use anyhow::*;
use async_trait::async_trait;

use clap::Parser;
use serde::Serialize;

/// A common result to be returned to users
pub type CliResult = Result<String, String>;

/// A common result to remove need for typing `Result<T, CliError>`
pub type CliTypedResult<T> = Result<T, Error>;

/// A common trait for all CLI commands to have consistent outputs
#[async_trait]
pub trait CliTool<T: Serialize + Send>: Sized + Send + Parser {
    /// Executes the command, returning a command specific type
    async fn execute(self) -> CliTypedResult<T>;

    /// Executes the command, and serializes it to the common JSON output type
    async fn execute_serialized(self) -> CliResult {
        to_common_result(self.execute().await).await
    }

    /// Executes the command, and throws away Ok(result) for the string Success
    async fn execute_serialized_success(self) -> CliResult {
        to_common_success_result(self.execute().await).await
    }

    /// Executes the main function.
    async fn execute_main() -> Result<()> {
        let tool = Self::parse();
        let result = tool.execute_serialized().await;
        match result {
            Result::Ok(val) => println!("{}", val),
            Result::Err(err) => {
                println!("{}", err);
                exit(1);
            }
        };
        Ok(())
    }
}

/// Convert any successful response to Success
pub async fn to_common_success_result<T>(result: Result<T>) -> CliResult {
    to_common_result(result.map(|_| "Success")).await
}

/// A result wrapper for displaying either a correct execution result or an error.
///
/// The purpose of this is to have a pretty easy to recognize JSON output format e.g.
///
/// ```json
/// {
///   "result":{
///     "encoded":{ ... }
///   }
/// }
///
/// {
///   "error": "Failed to run command"
/// }
/// ```
///
#[derive(Debug, Serialize)]
enum ResultWrapper<T> {
    #[serde(rename = "result")]
    Result(T),
    #[serde(rename = "error")]
    Error(String),
}

impl<T> From<CliTypedResult<T>> for ResultWrapper<T> {
    fn from(result: CliTypedResult<T>) -> Self {
        match result {
            CliTypedResult::Ok(inner) => ResultWrapper::Result(inner),
            CliTypedResult::Err(inner) => ResultWrapper::Error(inner.to_string()),
        }
    }
}

/// For pretty printing outputs in JSON
pub async fn to_common_result<T: Serialize>(result: Result<T>) -> CliResult {
    let is_err = result.is_err();
    let result: ResultWrapper<T> = result.into();
    let string = serde_json::to_string_pretty(&result)
        .map_err(|e| format!("could not serialize command output: {}", e))?;
    if is_err {
        CliResult::Err(string)
    } else {
        CliResult::Ok(string)
    }
}
