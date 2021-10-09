use crate::{error::CompileError, type_context::TypeContext};
use hir::{analysis::types::record_field_resolver, ir::*};
use std::collections::HashSet;

const RECORD_NAME: &str = "$record";

pub fn transform(
    update: &RecordUpdate,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let field_types = record_field_resolver::resolve(
        update.type_(),
        update.position(),
        type_context.types(),
        type_context.records(),
    )?;
    let position = update.position();

    Ok(Let::new(
        Some(RECORD_NAME.into()),
        Some(update.type_().clone()),
        update.record().clone(),
        RecordConstruction::new(
            update.type_().clone(),
            field_types
                .iter()
                .map(|field_type| field_type.name())
                .collect::<HashSet<_>>()
                .difference(
                    &update
                        .fields()
                        .iter()
                        .map(|field| field.name())
                        .collect(),
                )
                .map(|&name| {
                    RecordField::new(
                        name,
                        RecordDeconstruction::new(
                            update.type_().clone().into(),
                            Variable::new(RECORD_NAME, position.clone()),
                            name,
                            position.clone(),
                        ),
                        position.clone(),
                    )
                })
                .chain(update.fields().iter().cloned())
                .collect(),
            position.clone(),
        ),
        position.clone(),
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::types;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_record_update() {
        let record_type = types::Record::new("r", Position::fake());

        assert_eq!(
            transform(
                &RecordUpdate::new(
                    record_type.clone(),
                    Variable::new("r", Position::fake()),
                    vec![RecordField::new(
                        "y",
                        None::new(Position::fake()),
                        Position::fake()
                    )],
                    Position::fake()
                ),
                &TypeContext::dummy(
                    Default::default(),
                    vec![(
                        "r".into(),
                        vec![
                            types::RecordField::new("x", types::Number::new(Position::fake())),
                            types::RecordField::new("y", types::None::new(Position::fake()))
                        ]
                    )]
                    .into_iter()
                    .collect()
                )
            ),
            Ok(Let::new(
                Some(RECORD_NAME.into()),
                Some(record_type.clone().into()),
                Variable::new("r", Position::fake()),
                RecordConstruction::new(
                    record_type.clone(),
                    vec![
                        RecordField::new(
                            "x",
                            RecordDeconstruction::new(
                                Some(record_type.into()),
                                Variable::new(RECORD_NAME, Position::fake()),
                                "x",
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        RecordField::new("y", None::new(Position::fake()), Position::fake())
                    ],
                    Position::fake()
                ),
                Position::fake()
            )
            .into())
        );
    }
}
