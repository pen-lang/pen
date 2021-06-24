use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use crate::{
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn infer_types(module: &Module, type_context: &TypeContext) -> Result<Module, CompileError> {
    let variables = environment_creator::create_from_module(module);

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| infer_definition(definition, &variables, type_context))
            .collect::<Result<_, _>>()?,
    ))
}

fn infer_definition(
    definition: &Definition,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Definition, CompileError> {
    Ok(Definition::new(
        definition.name(),
        definition.original_name(),
        infer_lambda(definition.lambda(), variables, type_context)?,
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn infer_lambda(
    lambda: &Lambda,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Lambda, CompileError> {
    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        infer_expression(
            lambda.body(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    lambda
                        .arguments()
                        .iter()
                        .map(|argument| (argument.name().into(), argument.type_().clone())),
                )
                .collect(),
            type_context,
        )?,
        lambda.position().clone(),
    ))
}

fn infer_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let infer_expression =
        |expression, variables: &_| infer_expression(expression, variables, type_context);

    Ok(match expression {
        Expression::Call(call) => {
            let function = infer_expression(call.function(), variables)?;

            Call::new(
                function.clone(),
                call.arguments()
                    .iter()
                    .map(|argument| infer_expression(argument, variables))
                    .collect::<Result<_, _>>()?,
                Some(type_extractor::extract_from_expression(
                    &function,
                    variables,
                    type_context,
                )?),
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let then = infer_expression(if_.then(), variables)?;
            let else_ = infer_expression(if_.else_(), variables)?;

            If::new(
                infer_expression(if_.condition(), variables)?,
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfList(if_) => {
            let then = infer_expression(if_.then(), variables)?;
            let else_ = infer_expression(if_.else_(), variables)?;

            IfList::new(
                infer_expression(if_.argument(), variables)?,
                if_.first_name(),
                if_.rest_name(),
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => {
            let argument = infer_expression(if_.argument(), variables)?;
            let branches = if_
                .branches()
                .iter()
                .map(|branch| -> Result<_, CompileError> {
                    Ok(IfTypeBranch::new(
                        branch.type_().clone(),
                        infer_expression(
                            branch.expression(),
                            &variables
                                .clone()
                                .into_iter()
                                .chain(vec![(if_.name().into(), branch.type_().clone())])
                                .collect(),
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let else_ = if_
                .else_()
                .map(|expression| infer_expression(expression, variables))
                .transpose()?;

            IfType::new(
                if_.name(),
                argument,
                branches,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => infer_lambda(lambda, variables, type_context)?.into(),
        Expression::Let(let_) => {
            let bound_type = type_extractor::extract_from_expression(
                let_.bound_expression(),
                variables,
                type_context,
            )?;

            Let::new(
                let_.name().map(String::from),
                Some(bound_type.clone()),
                infer_expression(let_.bound_expression(), variables)?,
                infer_expression(
                    let_.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(let_.name().map(|name| (name.into(), bound_type)))
                        .collect(),
                )?,
                let_.position().clone(),
            )
            .into()
        }
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ListElement::Multiple(element) => {
                            ListElement::Multiple(infer_expression(element, variables)?)
                        }
                        ListElement::Single(element) => {
                            ListElement::Single(infer_expression(element, variables)?)
                        }
                    })
                })
                .collect::<Result<_, CompileError>>()?,
            list.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => match operation {
            Operation::Arithmetic(operation) => ArithmeticOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(operation) => BooleanOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Equality(operation) => {
                let lhs = infer_expression(operation.lhs(), variables)?;
                let rhs = infer_expression(operation.rhs(), variables)?;

                EqualityOperation::new(
                    Some(
                        types::Union::new(
                            type_extractor::extract_from_expression(&lhs, variables, type_context)?,
                            type_extractor::extract_from_expression(&rhs, variables, type_context)?,
                            operation.position().clone(),
                        )
                        .into(),
                    ),
                    operation.operator(),
                    lhs,
                    rhs,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Not(operation) => NotOperation::new(
                infer_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Order(operation) => OrderOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Try(_) => todo!(),
        },
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .elements()
                .iter()
                .map(|(key, element)| Ok((key.clone(), infer_expression(element, variables)?)))
                .collect::<Result<_, CompileError>>()?,
            construction.position().clone(),
        )
        .into(),
        Expression::RecordElement(element) => {
            let record = infer_expression(element.record(), variables)?;

            RecordElement::new(
                Some(type_extractor::extract_from_expression(
                    &record,
                    variables,
                    type_context,
                )?),
                record,
                element.element_name(),
                element.position().clone(),
            )
            .into()
        }
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            infer_expression(update.record(), variables)?,
            update
                .elements()
                .iter()
                .map(|(key, element)| Ok((key.clone(), infer_expression(element, variables)?)))
                .collect::<Result<_, CompileError>>()?,
            update.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            infer_expression(coercion.argument(), variables)?,
            coercion.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => expression.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hir_mir::{
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
        position::Position,
    };
    use pretty_assertions::assert_eq;

    fn infer_module(module: &Module) -> Module {
        infer_types(
            module,
            &TypeContext::new(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION),
        )
        .unwrap()
    }

    #[test]
    fn infer_empty_module() {
        infer_module(&Module::new(vec![], vec![], vec![], vec![]));
    }

    #[test]
    fn infer_call() {
        assert_eq!(
            infer_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("x", Position::dummy()),
                            vec![],
                            None,
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("x", Position::dummy()),
                            vec![],
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::dummy()),
                                    Position::dummy()
                                )
                                .into()
                            ),
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )
        );
    }

    #[test]
    fn infer_equality_operation() {
        assert_eq!(
            infer_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        EqualityOperation::new(
                            None,
                            EqualityOperator::Equal,
                            None::new(Position::dummy()),
                            None::new(Position::dummy()),
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        EqualityOperation::new(
                            Some(
                                types::Union::new(
                                    types::None::new(Position::dummy()),
                                    types::None::new(Position::dummy()),
                                    Position::dummy()
                                )
                                .into()
                            ),
                            EqualityOperator::Equal,
                            None::new(Position::dummy()),
                            None::new(Position::dummy()),
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )
        );
    }

    #[test]
    fn infer_let() {
        assert_eq!(
            infer_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
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
                )],
            )),
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Let::new(
                            Some("x".into()),
                            Some(types::None::new(Position::dummy()).into()),
                            None::new(Position::dummy()),
                            Variable::new("x", Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )
        );
    }

    #[test]
    fn infer_record_element() {
        let type_definition = TypeDefinition::new(
            "r",
            "",
            vec![types::RecordElement::new(
                "x",
                types::None::new(Position::dummy()),
            )],
            false,
            false,
            false,
            Position::dummy(),
        );

        assert_eq!(
            infer_module(&Module::new(
                vec![type_definition.clone()],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Record::new("r", Position::dummy())
                        )],
                        types::None::new(Position::dummy()),
                        RecordElement::new(
                            None,
                            Variable::new("x", Position::dummy()),
                            "x",
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )),
            Module::new(
                vec![type_definition],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Record::new("r", Position::dummy())
                        )],
                        types::None::new(Position::dummy()),
                        RecordElement::new(
                            Some(types::Record::new("r", Position::dummy()).into()),
                            Variable::new("x", Position::dummy()),
                            "x",
                            Position::dummy()
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            )
        );
    }
}
