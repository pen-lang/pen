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
                Some(type_extractor::extract_from_expression(&function, types)?),
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
                        type_extractor::extract_from_block(&then, types)?,
                        type_extractor::extract_from_block(&else_, types)?,
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
                        infer_block(
                            branch.block(),
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
                .map(|block| infer_block(block, variables))
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
                            .map(|alternative| alternative.block())
                            .chain(&else_)
                            .map(|block| type_extractor::extract_from_block(block, types))
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
        Expression::Boolean(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => expression.clone(),
        _ => todo!(),
    })
}

fn infer_block(
    block: &Block,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Block, CompileError> {
    let mut variables = variables.clone();
    let mut statements = vec![];

    for statement in block.statements() {
        let statement = infer_statement(statement, &variables, types)?;

        if let Some(name) = statement.name() {
            variables.insert(
                name.into(),
                statement
                    .type_()
                    .cloned()
                    .ok_or_else(|| CompileError::TypeNotInferred(statement.position().clone()))?,
            );
        }

        statements.push(statement);
    }

    Ok(Block::new(
        statements,
        infer_expression(block.expression(), &variables, types)?,
    ))
}

fn infer_statement(
    statement: &Statement,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, Type>,
) -> Result<Statement, CompileError> {
    let expression = infer_expression(statement.expression(), variables, types)?;

    Ok(Statement::new(
        statement.name().map(|string| string.into()),
        expression.clone(),
        Some(type_extractor::extract_from_expression(&expression, types)?),
        statement.position().clone(),
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
