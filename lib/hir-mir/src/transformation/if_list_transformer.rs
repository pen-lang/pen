use super::super::error::CompileError;
use crate::{downcast_compiler, type_context::TypeContext};
use hir::{
    analysis::types::type_equality_checker,
    ir::*,
    types::{self, Type},
};

const FIRST_REST_NAME: &str = "$firstRest";

pub fn transform(if_: &IfList, type_context: &TypeContext) -> Result<Expression, CompileError> {
    let configuration = type_context.list_type_configuration();
    let position = if_.position();

    let element_type = if_
        .type_()
        .ok_or_else(|| CompileError::TypeNotInferred(position.clone()))?;
    let any_list_type = types::Reference::new(&configuration.list_type_name, position.clone());
    let first_rest_type =
        types::Reference::new(&configuration.first_rest_type_name, position.clone());
    let none_type = types::None::new(position.clone());
    let any_type = Type::from(types::Any::new(position.clone()));
    let any_thunk_type = types::Function::new(vec![], any_type.clone(), position.clone());

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
                    {
                        let first = Call::new(
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
                        );

                        if type_equality_checker::check(
                            element_type,
                            &any_type,
                            type_context.types(),
                        )? {
                            Expression::from(first)
                        } else {
                            Let::new(
                                Some(if_.first_name().into()),
                                Some(any_thunk_type.clone().into()),
                                first,
                                Lambda::new(
                                    vec![],
                                    element_type.clone(),
                                    downcast_compiler::compile(
                                        &Call::new(
                                            Some(any_thunk_type.into()),
                                            Variable::new(if_.first_name(), position.clone()),
                                            vec![],
                                            position.clone(),
                                        )
                                        .into(),
                                        element_type,
                                    ),
                                    position.clone(),
                                ),
                                position.clone(),
                            )
                            .into()
                        }
                    },
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

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};

    #[test]
    fn transform_if_list_with_number_type() {
        insta::assert_debug_snapshot!(transform(
            &IfList::new(
                Some(types::Number::new(Position::fake()).into()),
                Variable::new("xs", Position::fake()),
                "x",
                "xs",
                None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            &TypeContext::dummy(Default::default(), Default::default()),
        ));
    }

    #[test]
    fn transform_if_list_with_any_type() {
        insta::assert_debug_snapshot!(transform(
            &IfList::new(
                Some(types::Any::new(Position::fake()).into()),
                Variable::new("xs", Position::fake()),
                "x",
                "xs",
                None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            &TypeContext::dummy(Default::default(), Default::default()),
        ));
    }
}
