use super::variable_renamer;
use crate::ir::*;
use fnv::FnvHashMap;

pub fn qualify(module: &Module, prefix: &str) -> Module {
    let names = module
        .foreign_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                prefix.to_owned() + declaration.name(),
            )
        })
        .chain(module.function_definitions().iter().map(|definition| {
            (
                definition.name().into(),
                prefix.to_owned() + definition.name(),
            )
        }))
        .collect::<FnvHashMap<_, _>>();

    variable_renamer::rename(
        &Module::new(
            module.type_definitions().to_vec(),
            module.type_aliases().to_vec(),
            module
                .foreign_declarations()
                .iter()
                .map(|declaration| {
                    ForeignDeclaration::new(
                        names[declaration.name()].clone(),
                        declaration.foreign_name(),
                        declaration.calling_convention(),
                        declaration.type_().clone(),
                        declaration.position().clone(),
                    )
                })
                .collect(),
            module.function_declarations().to_vec(),
            module
                .function_definitions()
                .iter()
                .map(|definition| {
                    FunctionDefinition::new(
                        names[definition.name()].clone(),
                        definition.original_name(),
                        definition.lambda().clone(),
                        definition.foreign_definition_configuration().cloned(),
                        definition.is_public(),
                        definition.position().clone(),
                    )
                })
                .collect(),
            module.position().clone(),
        ),
        &names,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_definition() {
        assert_eq!(
            qualify(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake()
                ),
                false
            )],)
        );
    }

    #[test]
    fn qualify_variable() {
        assert_eq!(
            qualify(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Variable::new("foo.x", Position::fake()),
                    Position::fake()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_argument() {
        assert_eq!(
            qualify(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "foo.x",
                Lambda::new(
                    vec![Argument::new("x", types::None::new(Position::fake()))],
                    types::None::new(Position::fake()),
                    Variable::new("x", Position::fake()),
                    Position::fake()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_statement() {
        assert_eq!(
            qualify(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Let::new(
                            Some("x".into()),
                            None,
                            None::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    Let::new(
                        Some("x".into()),
                        None,
                        None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake()
                ),
                false
            )],)
        );
    }
}
