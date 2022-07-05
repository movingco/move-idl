//! Generates the IDL for a function.

use anyhow::*;
use move_idl_types::{IDLArgument, IDLScriptFunction};
use move_model::{
    model::{FunctionEnv, GlobalEnv, ModuleEnv},
    ty,
};

use crate::{convert::get_idl_type_for_type, utils::normalize_doc_string};

pub fn generate_idl_for_function(
    env: &GlobalEnv,
    module_env: ModuleEnv,
    func: &FunctionEnv,
) -> anyhow::Result<IDLScriptFunction> {
    let symbol_pool = module_env.symbol_pool();
    let name = symbol_pool.string(func.get_name()).to_string();
    let doc = normalize_doc_string(func.get_doc());
    let ty_args = func
        .get_named_type_parameters()
        .iter()
        .map(|ty_param| symbol_pool.string(ty_param.0).to_string())
        .collect();
    let args = func
        .get_parameters()
        .iter()
        .filter(|param| match &param.1 {
            ty::Type::Primitive(ty::PrimitiveType::Signer) => false,
            ty::Type::Reference(false, inner) => {
                !matches!(&**inner, ty::Type::Primitive(ty::PrimitiveType::Signer))
            }
            _ => true,
        })
        .map(|param| {
            let ty = get_idl_type_for_type(env, &param.1)?;
            Ok(IDLArgument {
                name: symbol_pool.string(param.0).to_string(),
                ty,
            })
        })
        .collect::<anyhow::Result<_>>()?;

    // This is a script function, so no code. But we need to include the module ID
    Ok(IDLScriptFunction {
        name,
        doc,
        ty_args,
        args,
    })
}
