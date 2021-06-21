use super::variable_renamer;
use crate::hir::*;
use std::collections::HashMap;

pub fn qualify(module: &Module, prefix: &str) -> Module {
    let names = module
        .definitions()
        .iter()
        .map(|definition| {
            (
                definition.name().into(),
                prefix.to_owned() + definition.name(),
            )
        })
        .collect::<HashMap<_, _>>();

    variable_renamer::rename(
        &Module::new(
            module.type_definitions().to_vec(),
            module.type_aliases().to_vec(),
            module.declarations().to_vec(),
            module
                .definitions()
                .iter()
                .map(|definition| {
                    Definition::new(
                        names[definition.name()].clone(),
                        definition.original_name(),
                        definition.lambda().clone(),
                        definition.is_public(),
                        definition.position().clone(),
                    )
                })
                .collect(),
        ),
        &names,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn qualify_definition() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy())),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], None::new(Position::dummy())),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }

    #[test]
    fn qualify_variable() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], Variable::new("x", None, Position::dummy())),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], Variable::new("foo.x", None, Position::dummy())),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_argument() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", types::None::new(Position::dummy()))],
                            types::None::new(Position::dummy()),
                            Block::new(vec![], Variable::new("x", None, Position::dummy())),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "foo.x",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], Variable::new("x", None, Position::dummy())),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }

    #[test]
    fn do_not_qualify_variable_shadowed_by_statement() {
        assert_eq!(
            qualify(
                &Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::dummy()),
                            Block::new(
                                vec![Statement::new(
                                    Some("x".into()),
                                    None::new(Position::dummy()),
                                    None,
                                    Position::dummy(),
                                )],
                                Variable::new("x", None, Position::dummy())
                            ),
                            Position::dummy()
                        ),
                        false
                    )]
                ),
                "foo."
            ),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "foo.x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(
                            vec![Statement::new(
                                Some("x".into()),
                                None::new(Position::dummy()),
                                None,
                                Position::dummy(),
                            )],
                            Variable::new("x", None, Position::dummy())
                        ),
                        Position::dummy()
                    ),
                    false
                )]
            )
        );
    }
}
