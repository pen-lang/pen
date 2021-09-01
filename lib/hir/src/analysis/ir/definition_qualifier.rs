use super::variable_renamer;
use crate::ir::*;
use std::collections::HashMap;

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
        .chain(module.definitions().iter().map(|definition| {
            (
                definition.name().into(),
                prefix.to_owned() + definition.name(),
            )
        }))
        .collect::<HashMap<_, _>>();

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
            module.declarations().to_vec(),
            module
                .definitions()
                .iter()
                .map(|definition| {
                    Definition::new(
                        names[definition.name()].clone(),
                        definition.original_name(),
                        definition.lambda().clone(),
                        definition.is_foreign(),
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
        test,
        test::{DefinitionFake, ModuleFake},
        types,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_definition() {
        assert_eq!(
            qualify(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        None::new(test::position()),
                        test::position()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(test::position()),
                    None::new(test::position()),
                    test::position()
                ),
                false
            )],)
        );
    }

    #[test]
    fn qualify_variable() {
        assert_eq!(
            qualify(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        Variable::new("x", test::position()),
                        test::position()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(test::position()),
                    Variable::new("foo.x", test::position()),
                    test::position()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_argument() {
        assert_eq!(
            qualify(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(test::position()))],
                        types::None::new(test::position()),
                        Variable::new("x", test::position()),
                        test::position()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "foo.x",
                Lambda::new(
                    vec![Argument::new("x", types::None::new(test::position()))],
                    types::None::new(test::position()),
                    Variable::new("x", test::position()),
                    test::position()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_statement() {
        assert_eq!(
            qualify(
                &Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        Let::new(
                            Some("x".into()),
                            None,
                            None::new(test::position()),
                            Variable::new("x", test::position()),
                            test::position(),
                        ),
                        test::position()
                    ),
                    false
                )],),
                "foo."
            ),
            Module::empty().set_definitions(vec![Definition::fake(
                "foo.x",
                Lambda::new(
                    vec![],
                    types::None::new(test::position()),
                    Let::new(
                        Some("x".into()),
                        None,
                        None::new(test::position()),
                        Variable::new("x", test::position()),
                        test::position(),
                    ),
                    test::position()
                ),
                false
            )],)
        );
    }
}
