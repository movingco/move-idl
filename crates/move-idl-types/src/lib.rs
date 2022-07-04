//! Types for the Move IDL specification.

use errmap::{ErrorDescription, ErrorMapping};
use module_id::ModuleIdData;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// IDL error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDLError {
    /// The constant name of error e.g., ECANT_PAY_DEPOSIT
    pub name: String,
    /// The code description. This is generated from the doc comments on the constant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

impl From<&ErrorDescription> for IDLError {
    fn from(desc: &ErrorDescription) -> Self {
        IDLError {
            name: desc.code_name.clone(),
            doc: if desc.code_description.is_empty() {
                None
            } else {
                Some(desc.code_description.clone())
            },
        }
    }
}

/// A set of modules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDLPackage {
    /// Name of the package.
    pub name: String,
    /// Modules.
    pub modules: BTreeMap<String, IDLModule>,
    /// Aliases of addresses.
    pub aliases: BTreeMap<String, AccountAddress>,
    /// Dependent modules.
    pub dependencies: BTreeMap<String, IDLModule>,
    /// Error map.
    pub errors: ErrorMapping,
    /// All structs.
    pub structs: Vec<IDLStruct>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IDLStructTag {
    /// Module ID.
    pub module_id: ModuleIdData,
    pub name: String,
}

/// Simplified ABI for a Move module.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDLModule {
    /// The module name and address.
    pub module_id: ModuleIdData,
    /// Documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Functions.
    pub functions: Vec<IDLScriptFunction>,
    /// Structs.
    pub structs: Vec<IDLStruct>,
    /// Errors.
    pub errors: BTreeMap<u64, IDLError>,
}

/// A type represented in the IDL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IDLType {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    TypeParameter(u16),

    Tuple(Vec<IDLType>),
    Vector(Box<IDLType>),
    Struct(IDLStructTag),
}

/// An `Ability` classifies what operations are permitted for a given type
#[repr(u8)]
#[derive(Debug, Clone, Eq, Copy, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IDLAbility {
    /// Allows values of types with this ability to be copied, via CopyLoc or ReadRef
    Copy = 0x1,
    /// Allows values of types with this ability to be dropped, via Pop, WriteRef, StLoc, Eq, Neq,
    /// or if left in a local when Ret is invoked
    /// Technically also needed for numeric operations (Add, BitAnd, Shift, etc), but all
    /// of the types that can be used with those operations have Drop
    Drop = 0x2,
    /// Allows values of types with this ability to exist inside a struct in global storage
    Store = 0x4,
    /// Allows the type to serve as a key for global storage operations: MoveTo, MoveFrom, etc.
    Key = 0x8,
}

/// A struct.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IDLStruct {
    /// ID of the module.
    pub module_id: ModuleIdData,
    /// Name of the struct.
    pub name: String,
    /// Documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub fields: Vec<IDLField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<String>,
    /// Abilities.
    pub abilities: BTreeSet<IDLAbility>,
}

/// A struct.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IDLField {
    /// Name of the field.
    pub name: String,
    /// Documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Type of the IDL field.
    pub ty: IDLType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDLArgument {
    pub name: String,
    /// Type of the argument.
    pub ty: IDLType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDLScriptFunction {
    /// Name of the script function.
    pub name: String,
    /// Documentation for the script function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub ty_args: Vec<String>,
    pub args: Vec<IDLArgument>,
}
