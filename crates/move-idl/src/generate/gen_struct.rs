//! Generates the IDL for a struct.

use anyhow::*;
use move_binary_format::file_format::Ability;
use move_idl_types::{IDLAbility, IDLField, IDLStruct};
use move_model::model::{GlobalEnv, StructEnv};

use crate::{convert::get_idl_type_for_type, normalize_doc_string};

pub fn generate_idl_for_struct(env: &GlobalEnv, struct_env: &StructEnv) -> Result<IDLStruct> {
    let symbol_pool = env.symbol_pool();
    let fields = struct_env
        .get_fields()
        .map(|field| {
            Ok(IDLField {
                name: symbol_pool.string(field.get_name()).to_string(),
                doc: normalize_doc_string(field.get_doc()),
                ty: get_idl_type_for_type(env, &field.get_type())?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let type_params: Vec<String> = struct_env
        .get_named_type_parameters()
        .iter()
        .map(|tp| symbol_pool.string(tp.0).to_string())
        .collect();

    Ok(IDLStruct {
        module_id: struct_env.module_env.get_verified_module().self_id().into(),
        name: symbol_pool.string(struct_env.get_name()).to_string(),
        doc: normalize_doc_string(struct_env.get_doc()),
        fields,
        type_params,
        abilities: struct_env
            .get_abilities()
            .into_iter()
            .map(|a| match a {
                Ability::Copy => IDLAbility::Copy,
                Ability::Drop => IDLAbility::Drop,
                Ability::Store => IDLAbility::Store,
                Ability::Key => IDLAbility::Key,
            })
            .collect(),
    })
}
