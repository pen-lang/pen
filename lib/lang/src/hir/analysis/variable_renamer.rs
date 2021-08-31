
use super::variable_transformer;
use crate::hir::*;
use std::collections::HashMap;

pub fn rename(module: &Module, names: &HashMap<String, String>) -> Module {
    variable_transformer::transform(module, &|variable| {
        if let Some(name) = names.get(variable.name()) {
            Variable::new(name, variable.position().clone()).into()
        } else {
            variable.clone().into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;
    use position::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn rename_variable() {
        assert_eq!(
            rename(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Variable::new("x", Position::dummy()),
                        Position::dummy()
                    ),
                    false
                )],),
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Variable::new("foo.x", Position::dummy()),
                    Position::dummy()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_rename_variable_shadowed_by_argument() {
        let module = Module::empty().set_definitions(vec![Definition::without_source(
            "x",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::dummy()))],
                types::None::new(Position::dummy()),
                Variable::new("x", Position::dummy()),
                Position::dummy(),
            ),
            false,
        )]);

        assert_eq!(
            rename(
                &module,
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }

    #[test]
    fn do_not_rename_variable_shadowed_by_statement() {
        assert_eq!(
            rename(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Let::new(
                            Some("x".into()),
                            None,
                            None::new(Position::dummy()),
                            Variable::new("x", Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy()
                    ),
                    false
                )],),
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Let::new(
                        Some("x".into()),
                        None,
                        None::new(Position::dummy()),
                        Variable::new("x", Position::dummy()),
                        Position::dummy(),
                    ),
                    Position::dummy()
                ),
                false
            )],)
        );
    }

    #[test]
    fn do_not_rename_shadowed_variable_in_let() {
        let module = Module::empty().set_definitions(vec![Definition::without_source(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::dummy()),
                Let::new(
                    Some("x".into()),
                    None,
                    None::new(Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy(),
                ),
                Position::dummy(),
            ),
            false,
        )]);

        assert_eq!(
            rename(
                &module,
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }
}
