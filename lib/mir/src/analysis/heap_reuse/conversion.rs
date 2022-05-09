use super::{error::ReuseError, heap_block_set::HeapBlockSet};
use crate::{ir::*, types::Type};
use fnv::FnvHashMap;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn convert(module: &Module) -> Result<Module, ReuseError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(convert_definition)
            .collect::<Result<_, _>>()?,
    ))
}

fn convert_definition(definition: &FunctionDefinition) -> Result<FunctionDefinition, ReuseError> {
    let (expression, _) =
        convert_expression(definition.body(), &Default::default(), &Default::default())?;

    Ok(FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        expression,
        definition.result_type().clone(),
        definition.is_thunk(),
    ))
}

fn convert_expression(
    expression: &Expression,
    dropped_blocks: &HeapBlockSet,
    reused_blocks: &HeapBlockSet,
) -> Result<(Expression, HeapBlockSet), ReuseError> {
    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            let (lhs, reused_blocks) =
                convert_expression(operation.lhs(), dropped_blocks, reused_blocks)?;
            let (rhs, reused_blocks) =
                convert_expression(operation.rhs(), dropped_blocks, &reused_blocks)?;

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                reused_blocks,
            )
        }
        Expression::Call(call) => {
            let mut arguments = vec![];

            let (function, mut reused_blocks) =
                convert_expression(call.function(), dropped_blocks, reused_blocks)?;

            for argument in call.arguments() {
                let (argument, blocks) =
                    convert_expression(argument, dropped_blocks, &reused_blocks)?;

                arguments.push(argument);
                reused_blocks = blocks;
            }

            (
                Call::new(call.type_().clone(), function, arguments).into(),
                reused_blocks,
            )
        }
        Expression::Case(_) => todo!(),
        Expression::CloneVariables(clone) => {
            let (expression, reused_blocks) =
                convert_expression(clone.expression(), dropped_blocks, reused_blocks)?;

            (
                CloneVariables::new(clone.variables().clone(), expression).into(),
                reused_blocks,
            )
        }
        Expression::ComparisonOperation(operation) => {
            let (lhs, reused_blocks) =
                convert_expression(operation.lhs(), dropped_blocks, reused_blocks)?;
            let (rhs, reused_blocks) =
                convert_expression(operation.rhs(), dropped_blocks, &reused_blocks)?;

            (
                ComparisonOperation::new(operation.operator(), lhs, rhs).into(),
                reused_blocks,
            )
        }
        Expression::DropVariables(drop) => {
            let mut dropped_blocks = dropped_blocks.clone();

            for type_ in drop.variables().values() {
                if should_reuse_type(type_) {
                    dropped_blocks.add(type_);
                }
            }

            let (expression, mut reused_blocks) =
                convert_expression(drop.expression(), &dropped_blocks, reused_blocks)?;
            let mut variables = FnvHashMap::default();

            for (name, type_) in drop.variables() {
                if reused_blocks.remove(type_) {
                    variables.insert(
                        name.into(),
                        reuse_block(&dropped_blocks, &reused_blocks, type_).unwrap(),
                    );
                }
            }

            if variables.is_empty() {
                (
                    DropVariables::new(drop.variables().clone(), expression).into(),
                    reused_blocks,
                )
            } else {
                (
                    RetainVariables::new(
                        variables,
                        DropVariables::new(drop.variables().clone(), expression),
                    )
                    .into(),
                    reused_blocks,
                )
            }
        }
        Expression::If(_) => todo!(),
        Expression::Let(let_) => {
            let (bound_expression, reused_blocks) =
                convert_expression(let_.bound_expression(), dropped_blocks, reused_blocks)?;
            let (expression, reused_blocks) =
                convert_expression(let_.expression(), dropped_blocks, &reused_blocks)?;

            (
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    expression,
                )
                .into(),
                reused_blocks,
            )
        }
        Expression::LetRecursive(let_) => {
            let (expression, reused_blocks) =
                convert_expression(let_.expression(), dropped_blocks, &reused_blocks)?;

            (
                LetRecursive::new(convert_definition(let_.definition())?, expression).into(),
                reused_blocks,
            )
        }
        Expression::Record(record) => {
            let mut fields = vec![];
            let mut reused_blocks = reused_blocks.clone();

            for field in record.fields() {
                let (expression, blocks) =
                    convert_expression(field, dropped_blocks, &reused_blocks)?;

                fields.push(expression);
                reused_blocks = blocks;
            }

            if let Some(id) = reuse_block(
                dropped_blocks,
                &reused_blocks,
                &record.type_().clone().into(),
            ) {
                reused_blocks.add(&record.type_().clone().into());

                (
                    ReuseRecord::new(id, Record::new(record.type_().clone(), fields)).into(),
                    reused_blocks,
                )
            } else {
                (expression.clone(), reused_blocks)
            }
        }
        Expression::RecordField(field) => {
            let (expression, reused_blocks) =
                convert_expression(field.record(), dropped_blocks, reused_blocks)?;

            (
                RecordField::new(field.type_().clone(), field.index(), expression).into(),
                reused_blocks,
            )
        }
        Expression::ReuseRecord(_) | Expression::RetainVariables(_) => {
            return Err(ReuseError::ExpressionNotSupported)
        }
        Expression::TryOperation(_) => todo!(),
        Expression::Variant(variant) => {
            let (expression, reused_blocks) =
                convert_expression(variant.payload(), dropped_blocks, reused_blocks)?;

            (
                Variant::new(variant.type_().clone(), expression).into(),
                reused_blocks,
            )
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => (expression.clone(), reused_blocks.clone()),
    })
}

fn reuse_block(
    dropped_blocks: &HeapBlockSet,
    reused_blocks: &HeapBlockSet,
    type_: &Type,
) -> Option<String> {
    let count = dropped_blocks.get(type_) - reused_blocks.get(type_);

    if count > 0 {
        Some(get_block_id(type_, count))
    } else {
        None
    }
}

fn get_block_id(type_: &Type, count: usize) -> String {
    format!("{}-{}", get_type_id(type_), count)
}

fn get_type_id(type_: &Type) -> String {
    let mut hasher = DefaultHasher::new();

    type_.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}

fn should_reuse_type(type_: &Type) -> bool {
    matches!(type_, Type::Record(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;
    use pretty_assertions::assert_eq;

    fn reuse_in_expression(expression: &Expression) -> (Expression, HeapBlockSet) {
        convert_expression(expression, &Default::default(), &Default::default()).unwrap()
    }

    #[test]
    fn do_not_reuse_record() {
        let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into();

        assert_eq!(reuse_in_expression(&record), (record, Default::default()),);
    }

    #[test]
    fn reuse_record() {
        let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
        let block_id = get_block_id(&record.type_().clone().into(), 1);
        let drop = DropVariables::new(
            [("x".into(), record.type_().clone().into())]
                .into_iter()
                .collect(),
            record.clone(),
        );

        assert_eq!(
            reuse_in_expression(&drop.clone().into()),
            (
                RetainVariables::new(
                    [("x".into(), block_id.clone())].into_iter().collect(),
                    DropVariables::new(
                        drop.variables().clone(),
                        ReuseRecord::new(block_id, record),
                    ),
                )
                .into(),
                Default::default()
            ),
        );
    }

    #[test]
    fn reuse_nested_records() {
        let record = Record::new(
            types::Record::new("a"),
            vec![Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into()],
        );

        let outer_block_id = get_block_id(&record.type_().clone().into(), 1);
        let inner_block_id = get_block_id(&record.type_().clone().into(), 2);

        let drop = DropVariables::new(
            [
                ("x".into(), record.type_().clone().into()),
                ("y".into(), record.type_().clone().into()),
            ]
            .into_iter()
            .collect(),
            record.clone(),
        );

        assert_eq!(
            reuse_in_expression(&drop.clone().into()),
            (
                RetainVariables::new(
                    [
                        ("x".into(), outer_block_id.clone()),
                        ("y".into(), inner_block_id.clone())
                    ]
                    .into_iter()
                    .collect(),
                    DropVariables::new(
                        drop.variables().clone(),
                        ReuseRecord::new(
                            outer_block_id,
                            Record::new(
                                types::Record::new("a"),
                                vec![ReuseRecord::new(
                                    inner_block_id,
                                    Record::new(
                                        types::Record::new("a"),
                                        vec![Expression::Number(42.0)]
                                    )
                                )
                                .into()],
                            )
                        ),
                    ),
                )
                .into(),
                Default::default()
            ),
        );
    }
}
