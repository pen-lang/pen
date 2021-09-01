use super::super::{error::CompileError, list_type_configuration::ListTypeConfiguration};
use hir::types;
use hir::ir::*;

const FIRST_REST_NAME: &str = "$firstRest";

pub fn transform(
    if_: &IfList,
    configuration: &ListTypeConfiguration,
) -> Result<Expression, CompileError> {
    let position = if_.position();

    let element_type = if_
        .type_()
        .ok_or_else(|| CompileError::TypeNotInferred(position.clone()))?;
    let any_list_type = types::Reference::new(&configuration.list_type_name, position.clone());
    let first_rest_type =
        types::Reference::new(&configuration.first_rest_type_name, position.clone());
    let none_type = types::None::new(position.clone());
    let any_thunk_type =
        types::Function::new(vec![], types::Any::new(position.clone()), position.clone());

    Ok(IfType::new(
        FIRST_REST_NAME,
        Call::new(
            Some(
                types::Function::new(
                    vec![any_list_type.clone().into()],
                    types::Union::new(first_rest_type.clone(), none_type.clone(), position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.deconstruct_function_name, position.clone()),
            vec![if_.argument().clone()],
            position.clone(),
        ),
        vec![
            IfTypeBranch::new(
                first_rest_type.clone(),
                Let::new(
                    Some(if_.first_name().into()),
                    Some(
                        types::Function::new(vec![], element_type.clone(), position.clone()).into(),
                    ),
                    Let::new(
                        Some(if_.first_name().into()),
                        Some(any_thunk_type.clone().into()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![first_rest_type.clone().into()],
                                    any_thunk_type.clone(),
                                    position.clone(),
                                )
                                .into(),
                            ),
                            Variable::new(&configuration.first_function_name, position.clone()),
                            vec![Variable::new(FIRST_REST_NAME, position.clone()).into()],
                            position.clone(),
                        ),
                        Lambda::new(
                            vec![],
                            element_type.clone(),
                            IfType::new(
                                if_.first_name(),
                                Call::new(
                                    Some(any_thunk_type.into()),
                                    Variable::new(if_.first_name(), position.clone()),
                                    vec![],
                                    position.clone(),
                                ),
                                vec![IfTypeBranch::new(
                                    element_type.clone(),
                                    Variable::new(if_.first_name(), position.clone()),
                                )],
                                None,
                                position.clone(),
                            ),
                            position.clone(),
                        ),
                        position.clone(),
                    ),
                    Let::new(
                        Some(if_.rest_name().into()),
                        Some(any_list_type.clone().into()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![first_rest_type.into()],
                                    any_list_type,
                                    position.clone(),
                                )
                                .into(),
                            ),
                            Variable::new(&configuration.rest_function_name, position.clone()),
                            vec![Variable::new(FIRST_REST_NAME, position.clone()).into()],
                            position.clone(),
                        ),
                        if_.then().clone(),
                        position.clone(),
                    ),
                    position.clone(),
                ),
            ),
            IfTypeBranch::new(none_type, if_.else_().clone()),
        ],
        None,
        position.clone(),
    )
    .into())
}
