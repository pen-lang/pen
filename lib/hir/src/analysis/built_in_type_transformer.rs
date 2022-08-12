use super::type_transformer;
use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashSet;
use position::Position;

pub fn transform(module: &Module) -> Module {
    // This code should never be hit in the current implementation as all type
    // definitions' names have some kind of prefixes after conversion from AST
    // to HIR.
    let types = module
        .type_definitions()
        .iter()
        .map(|definition| definition.name())
        .chain(module.type_aliases().iter().map(|alias| alias.name()))
        .collect::<FnvHashSet<_>>();

    type_transformer::transform(module, |type_| match type_ {
        Type::Reference(reference) => {
            if types.contains(reference.name()) {
                type_.clone()
            } else {
                if let Some(type_) = built_in_type(reference.name(), reference.position()) {
                    type_.clone()
                } else {
                    type_.clone()
                }
            }
        }
        _ => type_.clone(),
    })
}

fn built_in_type(name: &str, position: &Position) -> Option<Type> {
    Some(match name {
        "any" => types::Any::new(position.clone()).into(),
        "boolean" => types::Boolean::new(position.clone()).into(),
        "error" => types::Error::new(position.clone()).into(),
        "none" => types::None::new(position.clone()).into(),
        "number" => types::Number::new(position.clone()).into(),
        "string" => types::ByteString::new(position.clone()).into(),
        _ => return None,
    })
}
