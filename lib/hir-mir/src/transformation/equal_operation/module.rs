use super::{function, operation};
use crate::{context::CompileContext, error::CompileError, transformation::collection_type};
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
    const LHS_NAME: &str = "$lhs";
    const RHS_NAME: &str = "$rhs";

    let position = type_.position();
    let name = function::transform_name(type_, context.types())?;

    Ok(FunctionDefinition::new(
        &name,
        &name,
        Lambda::new(
            vec![
                Argument::new(LHS_NAME, types::Any::new(position.clone())),
                Argument::new(RHS_NAME, types::Any::new(position.clone())),
            ],
            types::Boolean::new(position.clone()),
            IfType::new(
                LHS_NAME,
                Variable::new(LHS_NAME, position.clone()),
                vec![IfTypeBranch::new(
                    type_.clone(),
                    IfType::new(
                        RHS_NAME,
                        Variable::new(RHS_NAME, position.clone()),
                        vec![IfTypeBranch::new(
                            type_.clone(),
                            operation::transform(
                                context,
                                type_,
                                &Variable::new(LHS_NAME, position.clone()).into(),
                                &Variable::new(RHS_NAME, position.clone()).into(),
                                position,
                            )?,
                        )],
                        None,
                        position.clone(),
                    ),
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
