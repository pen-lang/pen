use super::type_transformer;
use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;
use position::Position;

pub fn replace(module: &Module) -> Module {
    let mut types = built_in_types();

    // This code should never be hit in the current implementation as all type
    // definitions' names have some kind of prefixes after conversion from AST
    // to HIR.
    for definition in module.type_definitions() {
        types.remove(definition.name());
    }

    type_transformer::transform(module, |type_| match type_ {
        Type::Reference(reference) => {
            if let Some(type_) = types.get(reference.name()) {
                type_.clone()
            } else {
                reference.clone().into()
            }
        }
        _ => type_.clone(),
    })
}

fn built_in_types() -> FnvHashMap<String, Type> {
    let position = Position::new("<built-in>", 1, 1, "");

    [
        ("any", types::Any::new(position.clone()).into()),
        ("boolean", types::Boolean::new(position.clone()).into()),
        ("error", types::Error::new(position.clone()).into()),
        ("none", types::None::new(position.clone()).into()),
        ("number", types::Number::new(position.clone()).into()),
        ("string", types::ByteString::new(position.clone()).into()),
    ]
    .into_iter()
    .map(|(name, type_)| (name.into(), type_))
    .collect()
}
