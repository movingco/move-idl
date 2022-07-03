//! Conversion utilities.

use anyhow::*;
use move_core_types::language_storage::TypeTag;
pub use move_idl_types::*;
use move_model::{
    model::GlobalEnv,
    ty::{self, Type},
};

/// Gets the [TypeTag] associated with a [ty::Type].
pub fn get_type_tag_for_type(move_type: &ty::Type) -> anyhow::Result<Option<TypeTag>> {
    use ty::Type::*;
    let tag = match move_type {
        Primitive(prim) => {
            use ty::PrimitiveType::*;
            match prim {
                Bool => TypeTag::Bool,
                U8 => TypeTag::U8,
                U64 => TypeTag::U64,
                U128 => TypeTag::U128,
                Address => TypeTag::Address,
                Signer => TypeTag::Signer,
                Num | Range | EventStore => {
                    bail!("Type {:?} is not allowed in scripts.", move_type)
                }
            }
        }
        Vector(ty) => {
            let tag = match get_type_tag_for_type(ty)? {
                Some(tag) => tag,
                None => return Ok(None),
            };
            TypeTag::Vector(Box::new(tag))
        }
        Tuple(_)
        | Struct(_, _, _)
        | TypeParameter(_)
        | Fun(_, _)
        | TypeDomain(_)
        | ResourceDomain(..)
        | Error
        | Var(_)
        | Reference(_, _) => return Ok(None),
    };
    Ok(Some(tag))
}

/// Gets the [IDLType] associated with a [Type].
pub fn get_idl_type_for_type(env: &GlobalEnv, ty: &Type) -> Result<IDLType> {
    Ok(match ty {
        Type::Primitive(pr) => match pr {
            ty::PrimitiveType::Bool => IDLType::Bool,
            ty::PrimitiveType::U8 => IDLType::U8,
            ty::PrimitiveType::U64 => IDLType::U64,
            ty::PrimitiveType::U128 => IDLType::U128,
            ty::PrimitiveType::Address => IDLType::Address,
            ty::PrimitiveType::Signer => IDLType::Signer,
            ty => bail!("unknown primitive {:?}", ty),
        },
        Type::Tuple(inner) => IDLType::Tuple(
            inner
                .iter()
                .map(|v| get_idl_type_for_type(env, v))
                .collect::<Result<Vec<_>>>()?,
        ),
        Type::Vector(inner) => IDLType::Vector(Box::new(get_idl_type_for_type(env, inner)?)),
        Type::Struct(_, _, _) => {
            let (struct_env, _) = ty
                .get_struct(env)
                .ok_or_else(|| anyhow!("struct could not be found"))?;
            IDLType::Struct(IDLStructTag {
                module_id: struct_env.module_env.get_verified_module().self_id().into(),
                name: struct_env
                    .get_name()
                    .display(struct_env.symbol_pool())
                    .to_string(),
            })
        }

        Type::TypeParameter(n) => IDLType::TypeParameter(*n),

        ty => bail!("could not process type {:?}", ty),
    })
}
