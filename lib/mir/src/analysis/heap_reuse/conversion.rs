use super::{error::ReuseError, heap_block_set::HeapBlockSet};
use crate::{ir::*, types::Type};
use fnv::{FnvHashMap, FnvHashSet};
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
    let (expression, _) = convert_expression(definition.body(), &Default::default())?;

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
) -> Result<(Expression, HeapBlockSet), ReuseError> {
    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            let (lhs, mut reused_blocks) = convert_expression(operation.lhs(), dropped_blocks)?;
            let (rhs, blocks) = convert_expression(operation.rhs(), dropped_blocks)?;

            reused_blocks.merge(&blocks);

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                reused_blocks,
            )
        }
        Expression::Call(call) => {
            let mut arguments = vec![];

            let (function, mut reused_blocks) =
                convert_expression(call.function(), dropped_blocks)?;

            for argument in call.arguments() {
                let (argument, blocks) = convert_expression(argument, dropped_blocks)?;

                arguments.push(argument);
                reused_blocks.merge(&blocks);
            }

            (
                Call::new(call.type_().clone(), function, arguments).into(),
                reused_blocks,
            )
        }
        Expression::Case(case) => {
            let (argument, mut reused_blocks) =
                convert_expression(case.argument(), dropped_blocks)?;
            let mut alternative_blocks = HeapBlockSet::default();
            let mut alternatives = vec![];

            for alternative in case.alternatives() {
                let (expression, blocks) =
                    convert_expression(alternative.expression(), dropped_blocks)?;

                alternative_blocks.max(&blocks);
                alternatives.push((expression, blocks));
            }

            let mut default_alternative = None;

            if let Some(alternative) = case.default_alternative() {
                let (expression, blocks) =
                    convert_expression(alternative.expression(), dropped_blocks)?;

                alternative_blocks.max(&blocks);
                default_alternative = Some((expression, blocks));
            }

            (
                Case::new(
                    argument,
                    alternatives
                        .into_iter()
                        .map(|(expression, blocks)| {
                            convert_branch(expression, &blocks, &alternative_blocks, dropped_blocks)
                        })
                        .zip(case.alternatives())
                        .map(|(expression, alternative)| {
                            Alternative::new(
                                alternative.type_().clone(),
                                alternative.name(),
                                expression,
                            )
                        })
                        .collect(),
                    default_alternative.and_then(|(expression, blocks)| {
                        case.default_alternative().map(|alternative| {
                            DefaultAlternative::new(
                                alternative.name(),
                                convert_branch(
                                    expression,
                                    &blocks,
                                    &alternative_blocks,
                                    dropped_blocks,
                                ),
                            )
                        })
                    }),
                )
                .into(),
                {
                    reused_blocks.merge(&alternative_blocks);
                    reused_blocks
                },
            )
        }
        Expression::CloneVariables(clone) => {
            let (expression, reused_blocks) =
                convert_expression(clone.expression(), dropped_blocks)?;

            (
                CloneVariables::new(clone.variables().clone(), expression).into(),
                reused_blocks,
            )
        }
        Expression::ComparisonOperation(operation) => {
            let (lhs, mut reused_blocks) = convert_expression(operation.lhs(), dropped_blocks)?;
            let (rhs, blocks) = convert_expression(operation.rhs(), dropped_blocks)?;

            reused_blocks.merge(&blocks);

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
                convert_expression(drop.expression(), &dropped_blocks)?;
            let mut variables = FnvHashMap::default();

            for (name, type_) in drop.variables() {
                if reused_blocks.get(type_) > 0 {
                    reused_blocks.remove(type_);

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
                    RetainHeap::new(
                        variables,
                        DropVariables::new(drop.variables().clone(), expression),
                    )
                    .into(),
                    reused_blocks,
                )
            }
        }
        Expression::If(if_) => {
            let (condition, mut reused_blocks) =
                convert_expression(if_.condition(), dropped_blocks)?;
            let (then_expression, then_blocks) = convert_expression(if_.then(), dropped_blocks)?;
            let (else_expression, else_blocks) = convert_expression(if_.else_(), dropped_blocks)?;

            let mut branch_blocks = then_blocks.clone();
            branch_blocks.max(&else_blocks);

            (
                If::new(
                    condition,
                    convert_branch(
                        then_expression,
                        &then_blocks,
                        &branch_blocks,
                        dropped_blocks,
                    ),
                    convert_branch(
                        else_expression,
                        &else_blocks,
                        &branch_blocks,
                        dropped_blocks,
                    ),
                )
                .into(),
                {
                    reused_blocks.merge(&branch_blocks);
                    reused_blocks
                },
            )
        }
        Expression::Let(let_) => {
            let (bound_expression, mut reused_blocks) =
                convert_expression(let_.bound_expression(), dropped_blocks)?;
            let (expression, blocks) = convert_expression(let_.expression(), dropped_blocks)?;

            reused_blocks.merge(&blocks);

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
                convert_expression(let_.expression(), dropped_blocks)?;

            (
                LetRecursive::new(convert_definition(let_.definition())?, expression).into(),
                reused_blocks,
            )
        }
        Expression::Record(record) => {
            let mut fields = vec![];
            let mut reused_blocks = HeapBlockSet::default();

            for field in record.fields() {
                let (expression, blocks) = convert_expression(field, dropped_blocks)?;

                fields.push(expression);
                reused_blocks.merge(&blocks);
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
            let (expression, reused_blocks) = convert_expression(field.record(), dropped_blocks)?;

            (
                RecordField::new(field.type_().clone(), field.index(), expression).into(),
                reused_blocks,
            )
        }
        Expression::RecordUpdate(update) => {
            let (record, mut reused_blocks) = convert_expression(update.record(), dropped_blocks)?;
            let mut fields = vec![];

            for field in update.fields() {
                let (expression, blocks) = convert_expression(field.expression(), dropped_blocks)?;

                fields.push(RecordUpdateField::new(field.index(), expression));
                reused_blocks.merge(&blocks);
            }

            (
                RecordUpdate::new(update.type_().clone(), record, fields).into(),
                reused_blocks,
            )
        }
        Expression::TryOperation(operation) => {
            let (operand, reused_blocks) = convert_expression(operation.operand(), dropped_blocks)?;

            (
                TryOperation::new(
                    operand,
                    operation.name(),
                    operation.type_().clone(),
                    // Ignore a then expression for simplicity.
                    operation.then().clone(),
                )
                .into(),
                reused_blocks,
            )
        }
        Expression::Variant(variant) => {
            let (expression, reused_blocks) =
                convert_expression(variant.payload(), dropped_blocks)?;

            (
                Variant::new(variant.type_().clone(), expression).into(),
                reused_blocks,
            )
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => (expression.clone(), Default::default()),
        Expression::DiscardHeap(_) | Expression::ReuseRecord(_) | Expression::RetainHeap(_) => {
            return Err(ReuseError::ExpressionNotSupported)
        }
    })
}

fn convert_branch(
    expression: Expression,
    branch_blocks: &HeapBlockSet,
    total_blocks: &HeapBlockSet,
    dropped_blocks: &HeapBlockSet,
) -> Expression {
    let difference = total_blocks
        .difference(branch_blocks)
        .collect::<FnvHashMap<_, _>>();

    if difference.is_empty() {
        expression
    } else {
        let mut reused_blocks = branch_blocks.clone();
        let mut ids = FnvHashSet::default();

        for (type_, count) in difference {
            for _ in 0..count {
                ids.insert(reuse_block(dropped_blocks, &reused_blocks, type_).unwrap());
                reused_blocks.add(type_);
            }
        }

        DiscardHeap::new(ids, expression).into()
    }
}

fn reuse_block(
    dropped_blocks: &HeapBlockSet,
    reused_blocks: &HeapBlockSet,
    type_: &Type,
) -> Option<String> {
    let count = dropped_blocks.get(type_) - reused_blocks.get(type_);

    if count > 0 {
        Some(get_id(type_, count))
    } else {
        None
    }
}

fn get_id(type_: &Type, count: usize) -> String {
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
        convert_expression(expression, &Default::default()).unwrap()
    }

    #[test]
    fn do_not_reuse_record() {
        let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into();

        assert_eq!(reuse_in_expression(&record), (record, Default::default()),);
    }

    #[test]
    fn reuse_record() {
        let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
        let id = get_id(&record.type_().clone().into(), 1);
        let drop = DropVariables::new(
            [("x".into(), record.type_().clone().into())]
                .into_iter()
                .collect(),
            record.clone(),
        );

        assert_eq!(
            reuse_in_expression(&drop.clone().into()),
            (
                RetainHeap::new(
                    [("x".into(), id.clone())].into_iter().collect(),
                    DropVariables::new(drop.variables().clone(), ReuseRecord::new(id, record)),
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

        let outer_id = get_id(&record.type_().clone().into(), 1);
        let inner_id = get_id(&record.type_().clone().into(), 2);

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
                RetainHeap::new(
                    [
                        ("x".into(), outer_id.clone()),
                        ("y".into(), inner_id.clone())
                    ]
                    .into_iter()
                    .collect(),
                    DropVariables::new(
                        drop.variables().clone(),
                        ReuseRecord::new(
                            outer_id,
                            Record::new(
                                types::Record::new("a"),
                                vec![ReuseRecord::new(
                                    inner_id,
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

    mod let_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn reuse_record() {
            let record_type = types::Record::new("a");
            let record = Record::new(record_type.clone(), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Let::new("y", record_type.clone(), record.clone(), Expression::None),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Let::new(
                                "y",
                                record_type,
                                ReuseRecord::new(id, record),
                                Expression::None
                            ),
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_with_nested_drops() {
            let record_type = types::Record::new("a");
            let record = Record::new(record_type.clone(), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Let::new(
                    "y",
                    record_type.clone(),
                    record.clone(),
                    DropVariables::new(
                        [("y".into(), record.type_().clone().into())]
                            .into_iter()
                            .collect(),
                        Expression::None,
                    ),
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Let::new(
                                "y",
                                record_type,
                                ReuseRecord::new(id, record.clone()),
                                DropVariables::new(
                                    [("y".into(), record.type_().clone().into())]
                                        .into_iter()
                                        .collect(),
                                    Expression::None,
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

    mod if_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn reuse_record() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                If::new(Expression::Boolean(true), record.clone(), record.clone()),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            If::new(
                                Expression::Boolean(true),
                                ReuseRecord::new(id.clone(), record.clone()),
                                ReuseRecord::new(id, record)
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_only_in_then() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                If::new(Expression::Boolean(true), record.clone(), Expression::None),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            If::new(
                                Expression::Boolean(true),
                                ReuseRecord::new(id.clone(), record),
                                DiscardHeap::new([id].into_iter().collect(), Expression::None)
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_only_in_else() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                If::new(Expression::Boolean(true), Expression::None, record.clone()),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            If::new(
                                Expression::Boolean(true),
                                DiscardHeap::new(
                                    [id.clone()].into_iter().collect(),
                                    Expression::None
                                ),
                                ReuseRecord::new(id, record),
                            )
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

            let outer_id = get_id(&record.type_().clone().into(), 1);
            let inner_id = get_id(&record.type_().clone().into(), 2);

            let drop = DropVariables::new(
                [
                    ("x".into(), record.type_().clone().into()),
                    ("y".into(), record.type_().clone().into()),
                ]
                .into_iter()
                .collect(),
                If::new(Expression::Boolean(true), record.clone(), record.clone()),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [
                            ("x".into(), outer_id.clone()),
                            ("y".into(), inner_id.clone())
                        ]
                        .into_iter()
                        .collect(),
                        DropVariables::new(drop.variables().clone(), {
                            let reuse = ReuseRecord::new(
                                outer_id,
                                Record::new(
                                    types::Record::new("a"),
                                    vec![ReuseRecord::new(
                                        inner_id,
                                        Record::new(
                                            types::Record::new("a"),
                                            vec![Expression::Number(42.0)],
                                        ),
                                    )
                                    .into()],
                                ),
                            );

                            If::new(Expression::Boolean(true), reuse.clone(), reuse)
                        }),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_nested_records_only_in_then() {
            let record = Record::new(
                types::Record::new("a"),
                vec![Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into()],
            );

            let outer_id = get_id(&record.type_().clone().into(), 1);
            let inner_id = get_id(&record.type_().clone().into(), 2);

            let drop = DropVariables::new(
                [
                    ("x".into(), record.type_().clone().into()),
                    ("y".into(), record.type_().clone().into()),
                ]
                .into_iter()
                .collect(),
                If::new(Expression::Boolean(true), record.clone(), Expression::None),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [
                            ("x".into(), outer_id.clone()),
                            ("y".into(), inner_id.clone())
                        ]
                        .into_iter()
                        .collect(),
                        DropVariables::new(drop.variables().clone(), {
                            let reuse = ReuseRecord::new(
                                outer_id.clone(),
                                Record::new(
                                    types::Record::new("a"),
                                    vec![ReuseRecord::new(
                                        inner_id.clone(),
                                        Record::new(
                                            types::Record::new("a"),
                                            vec![Expression::Number(42.0)],
                                        ),
                                    )
                                    .into()],
                                ),
                            );

                            If::new(
                                Expression::Boolean(true),
                                reuse,
                                DiscardHeap::new(
                                    [outer_id, inner_id].into_iter().collect(),
                                    Expression::None,
                                ),
                            )
                        }),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_nested_records_in_then_and_record_in_else() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let nested_record = Record::new(types::Record::new("a"), vec![record.clone().into()]);

            let outer_id = get_id(&nested_record.type_().clone().into(), 1);
            let inner_id = get_id(&nested_record.type_().clone().into(), 2);

            let drop = DropVariables::new(
                [
                    ("x".into(), nested_record.type_().clone().into()),
                    ("y".into(), nested_record.type_().clone().into()),
                ]
                .into_iter()
                .collect(),
                If::new(Expression::Boolean(true), nested_record.clone(), record),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [
                            ("x".into(), outer_id.clone()),
                            ("y".into(), inner_id.clone())
                        ]
                        .into_iter()
                        .collect(),
                        DropVariables::new(drop.variables().clone(), {
                            let reuse = ReuseRecord::new(
                                inner_id,
                                Record::new(
                                    types::Record::new("a"),
                                    vec![Expression::Number(42.0)],
                                ),
                            );

                            If::new(
                                Expression::Boolean(true),
                                ReuseRecord::new(
                                    outer_id.clone(),
                                    Record::new(
                                        types::Record::new("a"),
                                        vec![reuse.clone().into()],
                                    ),
                                ),
                                DiscardHeap::new([outer_id].into_iter().collect(), reuse),
                            )
                        }),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }
    }

    mod case {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn reuse_record_in_alternative() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(Type::None, "z", record.clone())],
                    None,
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![Alternative::new(
                                    Type::None,
                                    "z",
                                    ReuseRecord::new(id, record)
                                )],
                                None,
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_in_alternatives() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![
                        Alternative::new(Type::None, "z", record.clone()),
                        Alternative::new(Type::None, "z", record.clone()),
                    ],
                    None,
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![
                                    Alternative::new(
                                        Type::None,
                                        "z",
                                        ReuseRecord::new(id.clone(), record.clone())
                                    ),
                                    Alternative::new(Type::None, "z", ReuseRecord::new(id, record))
                                ],
                                None,
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_in_one_of_alternatives() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![
                        Alternative::new(Type::None, "z", record.clone()),
                        Alternative::new(Type::None, "z", Expression::None),
                    ],
                    None,
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![
                                    Alternative::new(
                                        Type::None,
                                        "z",
                                        ReuseRecord::new(id.clone(), record)
                                    ),
                                    Alternative::new(
                                        Type::None,
                                        "z",
                                        DiscardHeap::new(
                                            [id].into_iter().collect(),
                                            Expression::None
                                        )
                                    )
                                ],
                                None,
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_nested_records_in_alternatives() {
            let record = Record::new(
                types::Record::new("a"),
                vec![Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into()],
            );

            let outer_id = get_id(&record.type_().clone().into(), 1);
            let inner_id = get_id(&record.type_().clone().into(), 2);

            let drop = DropVariables::new(
                [
                    ("x".into(), record.type_().clone().into()),
                    ("y".into(), record.type_().clone().into()),
                ]
                .into_iter()
                .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![
                        Alternative::new(Type::None, "z", record.clone()),
                        Alternative::new(Type::None, "z", record.clone()),
                    ],
                    None,
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [
                            ("x".into(), outer_id.clone()),
                            ("y".into(), inner_id.clone())
                        ]
                        .into_iter()
                        .collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                {
                                    let reuse = ReuseRecord::new(
                                        outer_id,
                                        Record::new(
                                            types::Record::new("a"),
                                            vec![ReuseRecord::new(
                                                inner_id,
                                                Record::new(
                                                    types::Record::new("a"),
                                                    vec![Expression::Number(42.0)],
                                                ),
                                            )
                                            .into()],
                                        ),
                                    );

                                    vec![
                                        Alternative::new(Type::None, "z", reuse.clone()),
                                        Alternative::new(Type::None, "z", reuse),
                                    ]
                                },
                                None,
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_nested_records_only_in_one_of_alternatives() {
            let record = Record::new(
                types::Record::new("a"),
                vec![Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]).into()],
            );

            let outer_id = get_id(&record.type_().clone().into(), 1);
            let inner_id = get_id(&record.type_().clone().into(), 2);

            let drop = DropVariables::new(
                [
                    ("x".into(), record.type_().clone().into()),
                    ("y".into(), record.type_().clone().into()),
                ]
                .into_iter()
                .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![
                        Alternative::new(Type::None, "z", record.clone()),
                        Alternative::new(Type::None, "z", Expression::None),
                    ],
                    None,
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [
                            ("x".into(), outer_id.clone()),
                            ("y".into(), inner_id.clone())
                        ]
                        .into_iter()
                        .collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![
                                    Alternative::new(
                                        Type::None,
                                        "z",
                                        ReuseRecord::new(
                                            outer_id.clone(),
                                            Record::new(
                                                types::Record::new("a"),
                                                vec![ReuseRecord::new(
                                                    inner_id.clone(),
                                                    Record::new(
                                                        types::Record::new("a"),
                                                        vec![Expression::Number(42.0)],
                                                    ),
                                                )
                                                .into()],
                                            ),
                                        ),
                                    ),
                                    Alternative::new(
                                        Type::None,
                                        "z",
                                        DiscardHeap::new(
                                            [outer_id, inner_id].into_iter().collect(),
                                            Expression::None
                                        )
                                    ),
                                ],
                                None,
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_in_default_alternative() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![],
                    Some(DefaultAlternative::new("z", record.clone())),
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![],
                                Some(DefaultAlternative::new("z", ReuseRecord::new(id, record))),
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_only_in_default_alternative() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(Type::None, "z", Expression::None)],
                    Some(DefaultAlternative::new("z", record.clone())),
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![Alternative::new(
                                    Type::None,
                                    "z",
                                    DiscardHeap::new(
                                        [id.clone()].into_iter().collect(),
                                        Expression::None
                                    )
                                )],
                                Some(DefaultAlternative::new("z", ReuseRecord::new(id, record))),
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }

        #[test]
        fn reuse_record_not_in_default_alternative() {
            let record = Record::new(types::Record::new("a"), vec![Expression::Number(42.0)]);
            let id = get_id(&record.type_().clone().into(), 1);
            let drop = DropVariables::new(
                [("x".into(), record.type_().clone().into())]
                    .into_iter()
                    .collect(),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(Type::None, "z", record.clone())],
                    Some(DefaultAlternative::new("z", Expression::None)),
                ),
            );

            assert_eq!(
                reuse_in_expression(&drop.clone().into()),
                (
                    RetainHeap::new(
                        [("x".into(), id.clone())].into_iter().collect(),
                        DropVariables::new(
                            drop.variables().clone(),
                            Case::new(
                                Variable::new("y"),
                                vec![Alternative::new(
                                    Type::None,
                                    "z",
                                    ReuseRecord::new(id.clone(), record)
                                )],
                                Some(DefaultAlternative::new(
                                    "z",
                                    DiscardHeap::new([id].into_iter().collect(), Expression::None)
                                )),
                            )
                        ),
                    )
                    .into(),
                    Default::default()
                ),
            );
        }
    }
}
