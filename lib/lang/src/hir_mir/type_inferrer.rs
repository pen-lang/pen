use super::{environment_creator, type_extractor, union_type_creator, CompileError};
use crate::{
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn infer_types(module: &Module, types: &HashMap<String, Type>) -> Result<Module, CompileError> {
    let variables = environment_creator::create_from_module(module);

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| infer_definition(definition, &variables, types))
            .collect::<Result<_, _>>()?,
    ))
}

fn infer_definition(
    definition: &Definition,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Definition, CompileError> {
    Ok(Definition::new(
        definition.name(),
        definition.original_name(),
        infer_lambda(definition.lambda(), variables, types)?,
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn infer_lambda(
    lambda: &Lambda,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
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
            types,
        )?,
        lambda.position().clone(),
    ))
}

fn infer_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Expression, CompileError> {
    let infer_expression =
        |expression, variables: &_| infer_expression(expression, variables, types);

    Ok(match expression {
        Expression::Call(call) => {
            let function = infer_expression(call.function(), variables)?;

            Call::new(
                function.clone(),
                call.arguments()
                    .iter()
                    .map(|argument| infer_expression(argument, variables))
                    .collect::<Result<_, _>>()?,
                Some(type_extractor::extract_from_expression(&function, types)?),
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let then = infer_expression(if_.then(), variables)?;
            let else_ = infer_expression(if_.else_(), variables)?;

            If::new(
                infer_expression(if_.condition(), variables)?,
                then.clone(),
                else_.clone(),
                Some(
                    types::Union::new(
                        type_extractor::extract_from_expression(&then, types)?,
                        type_extractor::extract_from_expression(&else_, types)?,
                        if_.position().clone(),
                    )
                    .into(),
                ),
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
                then.clone(),
                else_.clone(),
                Some(
                    types::Union::new(
                        type_extractor::extract_from_expression(&then, types)?,
                        type_extractor::extract_from_expression(&else_, types)?,
                        if_.position().clone(),
                    )
                    .into(),
                ),
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
                argument.clone(),
                Some(type_extractor::extract_from_expression(&argument, types)?),
                branches.clone(),
                else_.clone(),
                Some(
                    union_type_creator::create_union_type(
                        &branches
                            .iter()
                            .map(|alternative| alternative.expression())
                            .chain(&else_)
                            .map(|expression| {
                                type_extractor::extract_from_expression(expression, types)
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        if_.position(),
                    )
                    .unwrap(),
                ),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => infer_lambda(lambda, variables, types)?.into(),
        Expression::Let(_) => todo!(),
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
                            type_extractor::extract_from_expression(&lhs, types)?,
                            type_extractor::extract_from_expression(&rhs, types)?,
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
                Some(type_extractor::extract_from_expression(&record, types)?),
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

    #[test]
    fn infer_empty_module() -> Result<(), CompileError> {
        infer_types(
            &Module::new(vec![], vec![], vec![], vec![]),
            &Default::default(),
        )?;

        Ok(())
    }
}
