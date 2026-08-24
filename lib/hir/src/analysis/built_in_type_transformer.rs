use super::type_transformer;
use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashSet;

// We replace reference types with built-in types if they are not overridden.
//
// Although another approach might be to append built-in type definitions, it
// would provide slightly worse code position information.
pub fn transform(module: &Module) -> Module {
    let types = module
        .type_definitions()
        .iter()
        .map(|definition| definition.name())
        .chain(module.type_aliases().iter().map(|alias| alias.name()))
        .collect::<FnvHashSet<_>>();

    type_transformer::transform(module, |type_| match type_ {
        Type::Reference(reference) => {
            if types.contains(reference.name()) {
                // This code should never be hit in the current implementation as all type
                // definitions' names have some kind of prefixes after conversion from AST
                // to HIR.
                type_.clone()
            } else {
                let position = reference.position();

                match reference.name() {
                    "any" => types::Any::new(position.clone()).into(),
                    "boolean" => types::Boolean::new(position.clone()).into(),
                    "error" => types::Error::new(position.clone()).into(),
                    "none" => types::None::new(position.clone()).into(),
                    "number" => types::Number::new(position.clone()).into(),
                    "string" => types::ByteString::new(position.clone()).into(),
                    _ => type_.clone(),
                }
            }
        }
        _ => type_.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{FunctionDefinitionFake, ModuleFake, TypeDefinitionFake};
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_empty_module() {
        assert_eq!(transform(&Module::empty()), Module::empty());
    }

    #[test]
    fn transform_function_result_type() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Reference::new("none", Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
            )])
        );
    }

    #[test]
    fn transform_function_argument_type() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "y",
                            types::Reference::new("none", Position::fake())
                        )],
                        types::Reference::new("foo", Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![Argument::new("y", types::None::new(Position::fake()))],
                    types::Reference::new("foo", Position::fake()),
                    None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
            )])
        );
    }

    #[test]
    fn transform_record_field() {
        assert_eq!(
            transform(
                &Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                    "x",
                    vec![types::RecordField::new(
                        "x",
                        types::Reference::new("none", Position::fake()),
                    )],
                    false,
                    false,
                    false
                )])
            ),
            Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                "x",
                vec![types::RecordField::new(
                    "x",
                    types::None::new(Position::fake()),
                )],
                false,
                false,
                false
            )])
        );
    }
}
