use std::collections::BTreeMap;

use errmap::{ErrorDescription, ErrorMapping};
use module_id::ModuleIdData;
use serde::{Deserialize, Serialize};

/// IDL error mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDLErrorMapping {
    /// The set of error categories and their descriptions
    pub error_categories: BTreeMap<u64, IDLError>,
    /// The set of modules, and the module-specific errors
    pub module_error_maps: BTreeMap<ModuleIdData, BTreeMap<u64, IDLError>>,
}

impl From<ErrorMapping> for IDLErrorMapping {
    fn from(errmap: ErrorMapping) -> Self {
        IDLErrorMapping {
            error_categories: errmap
                .error_categories
                .into_iter()
                .map(|(k, v)| (k, IDLError::from(v)))
                .collect(),
            module_error_maps: errmap
                .module_error_maps
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.into_iter().map(|(k, v)| (k, IDLError::from(v))).collect(),
                    )
                })
                .collect(),
        }
    }
}

/// IDL error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDLError {
    /// The constant name of error e.g., ECANT_PAY_DEPOSIT
    pub name: String,
    /// The code description. This is generated from the doc comments on the constant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

impl From<ErrorDescription> for IDLError {
    fn from(desc: ErrorDescription) -> Self {
        IDLError {
            name: desc.code_name,
            doc: if desc.code_description.is_empty() {
                None
            } else {
                Some(desc.code_description)
            },
        }
    }
}
