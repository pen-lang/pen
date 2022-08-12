use super::variable_transformer;
use crate::ir::*;

pub fn transform(module: &Module) -> Module {
    variable_transformer::transform(module, &|variable| match variable.name() {
        "debug" => {
            BuiltInFunction::new(BuiltInFunctionName::Debug, variable.position().clone()).into()
        }
        "error" => {
            BuiltInFunction::new(BuiltInFunctionName::Error, variable.position().clone()).into()
        }
        "false" => Boolean::new(false, variable.position().clone()).into(),
        "go" => {
            BuiltInFunction::new(BuiltInFunctionName::Spawn, variable.position().clone()).into()
        }
        "none" => None::new(variable.position().clone()).into(),
        "race" => {
            BuiltInFunction::new(BuiltInFunctionName::Race, variable.position().clone()).into()
        }
        "size" => {
            BuiltInFunction::new(BuiltInFunctionName::Size, variable.position().clone()).into()
        }
        "source" => {
            BuiltInFunction::new(BuiltInFunctionName::Source, variable.position().clone()).into()
        }
        "true" => Boolean::new(true, variable.position().clone()).into(),
        _ => variable.clone().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};
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
                        types::None::new(Position::fake()),
                        Variable::new("none", Position::fake()),
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
}
