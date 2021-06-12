use super::{type_extraction, CompileError};
use crate::{
    compile::union_types,
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn infer_types(module: &Module, types: &HashMap<String, Type>) -> Result<Module, CompileError> {
    let variables = module
        .declarations()
        .iter()
        .map(|declaration| (declaration.name().into(), declaration.type_().clone()))
        .chain(
            module
                .definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.type_().clone())),
        )
        .collect();

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
        infer_expression(definition.body(), variables, types)?,
        definition.type_().clone(),
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn infer_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Expression, CompileError> {
    let infer_expression = |expression, variables| infer_expression(expression, variables, types);
    let infer_block = |block, variables: &HashMap<_, _>| infer_block(block, variables, types);

    Ok(match expression {
        Expression::Call(call) => {
            let function = infer_expression(call.function(), variables)?;

            Call::new(
                function.clone(),
                call.arguments()
                    .iter()
                    .map(|argument| infer_expression(argument, variables))
                    .collect::<Result<_, _>>()?,
                Some(type_extraction::extract_from_expression(&function, types)?),
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let condition = infer_expression(if_.condition(), variables)?;
            let then = infer_block(if_.then(), variables)?;
            let else_ = infer_block(if_.else_(), variables)?;

            If::new(
                condition,
                then.clone(),
                else_.clone(),
                Some(
                    types::Union::new(
                        type_extraction::extract_from_block(&then, types)?,
                        type_extraction::extract_from_block(&else_, types)?,
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
            let alternatives = if_
                .alternatives()
                .iter()
                .map(|alternative| -> Result<_, CompileError> {
                    Ok(Alternative::new(
                        alternative.type_().clone(),
                        infer_block(
                            alternative.block(),
                            &variables
                                .clone()
                                .into_iter()
                                .chain(vec![(if_.name().into(), alternative.type_().clone())])
                                .collect(),
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let default_alternative = if_
                .default_alternative()
                .map(|alternative| infer_block(alternative, variables))
                .transpose()?;

            IfType::new(
                if_.name(),
                argument.clone(),
                Some(type_extraction::extract_from_expression(&argument, types)?),
                alternatives.clone(),
                default_alternative.clone(),
                Some(
                    union_types::create_union_type(
                        &alternatives
                            .iter()
                            .map(|alternative| alternative.block())
                            .chain(&default_alternative)
                            .map(|block| type_extraction::extract_from_block(block, types))
                            .collect::<Result<Vec<_>, _>>()?,
                        if_.position(),
                    )
                    .unwrap(),
                ),
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => Lambda::new(
            lambda.arguments().to_vec(),
            infer_block(
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
            )?,
            lambda.type_().clone(),
            lambda.position().clone(),
        )
        .into(),
        Expression::Boolean(_) | Expression::Number(_) => expression.clone(),
        _ => todo!(),
    })
}

fn infer_block(
    block: &Block,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Block, CompileError> {
    let mut variables = variables.clone();
    let mut assignments = vec![];

    for assignment in block.assignments() {
        let assignment = infer_assignment(assignment, &variables, types)?;

        variables.insert(assignment.name().into(), assignment.type_().clone());
        assignments.push(assignment);
    }

    Ok(Block::new(
        assignments,
        infer_expression(block.expression(), &variables, types)?,
    ))
}

fn infer_assignment(
    assignment: &Assignment,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Assignment, CompileError> {
    let expression = infer_expression(assignment.expression(), variables, types)?;

    Ok(Assignment::new(
        assignment.name(),
        expression.clone(),
        type_extraction::extract_from_expression(&expression, types)?,
        assignment.position().clone(),
    ))
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
