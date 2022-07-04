//! Generates the IDL for a module.

use anyhow::*;
use errmapgen::{ErrorDescription, ErrorMapping};
use module_id::ModuleIdData;
use move_bytecode_verifier::script_signature;
use move_core_types::identifier::IdentStr;
use move_idl_types::{IDLModule, IDLStruct};
use move_model::{
    model::{FunctionVisibility, GlobalEnv, ModuleEnv},
    ty,
};
use std::collections::BTreeMap;

use move_idl_types::IDLError;

use crate::{convert::get_type_tag_for_type, normalize_doc_string};

use super::{gen_function::generate_idl_for_function, gen_struct::generate_idl_for_struct};

/// Compute the IDLs of all script functions in a module.
pub fn generate_idl_for_module(
    env: &GlobalEnv,
    error_mapping: &ErrorMapping,
    module_env: ModuleEnv,
) -> anyhow::Result<IDLModule> {
    // Get all the script functions in this module
    let script_iter: Vec<_> = module_env
        .get_functions()
        .filter(|func| {
            let module = module_env.get_verified_module();
            let func_name = module_env.symbol_pool().string(func.get_name());
            let func_ident = IdentStr::new(&func_name).unwrap();
            // only pick up script functions that also have a script-callable signature.
            // and check all arguments have a valid type tag
            func.visibility() == FunctionVisibility::Script
                && script_signature::verify_module_function_signature_by_name(
                    module,
                    func_ident,
                    script_signature::no_additional_script_signature_checks,
                )
                .is_ok()
                && func
                    .get_parameters()
                    .iter()
                    .skip_while(|param| match &param.1 {
                        ty::Type::Primitive(ty::PrimitiveType::Signer) => true,
                        ty::Type::Reference(_, inner) => {
                            matches!(&**inner, ty::Type::Primitive(ty::PrimitiveType::Signer))
                        }
                        _ => false,
                    })
                    .all(|param| {
                        matches!(
                            get_type_tag_for_type(&param.1),
                            Err(_) | Result::Ok(Some(_))
                        )
                    })
                && func.get_return_count() == 0
        })
        .collect();

    let structs = generate_idl_structs_for_module(env, module_env.clone())?;

    let mut functions = Vec::new();
    for func in &script_iter {
        functions.push(generate_idl_for_function(env, module_env.clone(), func)?);
    }

    let module_id = module_env.get_verified_module().self_id().into();

    Ok(IDLModule {
        module_id,
        doc: normalize_doc_string(module_env.get_doc()),
        functions,
        structs,
        errors: generate_idl_errors_for_module(error_mapping, &module_env)?,
    })
}

pub fn generate_idl_structs_for_module(
    env: &GlobalEnv,
    module_env: ModuleEnv,
) -> Result<Vec<IDLStruct>> {
    let mut ret = vec![];
    for struct_env in module_env.into_structs() {
        let idl_struct = generate_idl_for_struct(env, &struct_env)?;
        ret.push(idl_struct)
    }
    Ok(ret)
}

/// Generates the error map for the module.
pub fn generate_idl_errors_for_module(
    error_mapping: &ErrorMapping,
    module_env: &ModuleEnv,
) -> Result<BTreeMap<u64, IDLError>> {
    let module_id: ModuleIdData = module_env.get_verified_module().self_id().into();
    let result: Option<&BTreeMap<u64, ErrorDescription>> =
        error_mapping.module_error_maps.get(&module_id);

    Ok(match result {
        Some(result) => result
            .iter()
            .map(|(k, v)| -> (u64, IDLError) { (*k, v.into()) })
            .collect(),
        None => BTreeMap::new(),
    })
}
