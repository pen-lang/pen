use super::variable_transformer;
use crate::ir::*;
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
    use crate::{test, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn rename_variable() {
        assert_eq!(
            rename(
                &Module::empty().set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        Variable::new("x", test::position()),
                        test::position()
                    ),
                    false
                )],),
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_definitions(vec![Definition::without_source(
                "x",
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
    fn do_not_rename_variable_shadowed_by_argument() {
        let module = Module::empty().set_definitions(vec![Definition::without_source(
            "x",
            Lambda::new(
                vec![Argument::new("x", types::None::new(test::position()))],
                types::None::new(test::position()),
                Variable::new("x", test::position()),
                test::position(),
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
                &vec![("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_definitions(vec![Definition::without_source(
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
            )],)
        );
    }

    #[test]
    fn do_not_rename_shadowed_variable_in_let() {
        let module = Module::empty().set_definitions(vec![Definition::without_source(
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
                test::position(),
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
