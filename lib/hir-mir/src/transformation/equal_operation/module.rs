use super::expression::transform_equal_operation;
use super::function;
use crate::context::CompileContext;
use crate::error::CompileError;
use fnv::FnvHashSet;
use hir::{
    analysis::{expression_visitor, type_canonicalizer, type_visitor, AnalysisError},
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
                collect_parameter_types(context, module)?
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
                            transform_equal_operation(
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

fn collect_parameter_types(
    context: &CompileContext,
    module: &Module,
) -> Result<FnvHashSet<Type>, AnalysisError> {
    let mut types = FnvHashSet::default();

    type_visitor::visit(module, |type_| match type_ {
        Type::List(list_type) => {
            types.insert(list_type.element());
        }
        Type::Map(map_type) => {
            types.extend([map_type.key(), map_type.value()]);
        }
        _ => {}
    });

    expression_visitor::visit(module, |expression| match expression {
        Expression::IfList(if_) => {
            types.extend(if_.type_());
        }
        Expression::IfMap(if_) => {
            types.extend([if_.key_type(), if_.value_type()].into_iter().flatten());
        }
        Expression::List(list) => {
            types.insert(list.type_());
        }
        Expression::Map(map) => {
            types.extend([map.key_type(), map.value_type()]);
        }
        _ => {}
    });

    types
        .iter()
        .map(|type_| type_canonicalizer::canonicalize(type_, context.types()))
        .collect::<Result<_, _>>()
}
