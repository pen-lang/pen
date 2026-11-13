use super::expression;
use crate::{
    CompileError,
    context::Context,
    transformation::{collection_type, hash_calculation::function},
};
use hir::{
    ir::*,
    types::{self, Type},
};

pub fn transform(context: &Context, module: &Module) -> Result<Module, CompileError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(
                collection_type::collect_comparable_parameter_types(context, module)?
                    .into_iter()
                    .map(|type_| compile_function_definition(context, &type_))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .collect(),
        module.position().clone(),
    ))
}

fn compile_function_definition(
    context: &Context,
    type_: &Type,
) -> Result<FunctionDefinition, CompileError> {
    const ARGUMENT_NAME: &str = "$x";

    let position = type_.position();
    let name = function::transform_name(type_, context.types())?;

    Ok(FunctionDefinition::new(
        &name,
        &name,
        Lambda::new(
            vec![Argument::new(
                ARGUMENT_NAME,
                types::Any::new(position.clone()),
            )],
            types::Number::new(position.clone()),
            IfType::new(
                ARGUMENT_NAME,
                Variable::new(ARGUMENT_NAME, position.clone()),
                vec![IfTypeBranch::new(
                    type_.clone(),
                    expression::transform(
                        context,
                        &Variable::new(ARGUMENT_NAME, position.clone()).into(),
                        type_,
                        position,
                    )?,
                )],
                None,
                position.clone(),
            ),
            position.clone(),
        ),
        None,
        false,
        position.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::{ModuleFake, TypeAliasFake};
    use position::{Position, test::PositionFake};

    fn transform_module(module: &Module) -> Result<Module, CompileError> {
        transform(
            &Context::new(module, Some(COMPILE_CONFIGURATION.clone())),
            module,
        )
    }

    #[test]
    fn transform_comparable_type() {
        let module = Module::empty().set_type_aliases(vec![TypeAlias::fake(
            "a",
            types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            ),
            false,
            false,
        )]);

        insta::assert_debug_snapshot!(transform_module(&module));
    }

    #[test]
    fn do_not_transform_any() {
        let module = Module::empty().set_type_aliases(vec![TypeAlias::fake(
            "a",
            types::List::new(types::Any::new(Position::fake()), Position::fake()),
            false,
            false,
        )]);

        assert_eq!(transform_module(&module), Ok(module.clone()));
    }

    #[test]
    fn do_not_transform_function() {
        let module = Module::empty().set_type_aliases(vec![TypeAlias::fake(
            "a",
            types::List::new(
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake()),
                Position::fake(),
            ),
            false,
            false,
        )]);

        assert_eq!(transform_module(&module), Ok(module.clone()));
    }
}
