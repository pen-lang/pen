use super::expression;
use crate::{
    context::CompileContext,
    transformation::{collection_type, hash_calculation::function},
    CompileError,
};
use hir::{
    ir::*,
    types::{self, Type},
};

pub fn transform(context: &CompileContext, module: &Module) -> Result<Module, CompileError> {
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
    context: &CompileContext,
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
