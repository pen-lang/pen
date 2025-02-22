use super::variable_transformer;
use crate::ir::*;
use fnv::FnvHashMap;

pub fn rename(module: &Module, names: &FnvHashMap<String, String>) -> Module {
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
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn rename_variable() {
        assert_eq!(
            rename(
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
                &[("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
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
    fn do_not_rename_variable_shadowed_by_argument() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "x",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::fake()))],
                types::None::new(Position::fake()),
                Variable::new("x", Position::fake()),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            rename(
                &module,
                &[("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }

    #[test]
    fn do_not_rename_variable_shadowed_by_statement() {
        assert_eq!(
            rename(
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
                &[("x".into(), "foo.x".into())].into_iter().collect()
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
            )],)
        );
    }

    #[test]
    fn do_not_rename_shadowed_variable_in_let() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(
            rename(
                &module,
                &[("x".into(), "foo.x".into())].into_iter().collect()
            ),
            module
        );
    }
}
