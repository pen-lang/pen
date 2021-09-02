use crate::{error::CompileError, type_context::TypeContext};
use hir::{analysis::types::record_element_resolver, ir::*};
use std::collections::HashSet;

const RECORD_NAME: &str = "$record";

pub fn transform(
    update: &RecordUpdate,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    let element_types = record_element_resolver::resolve(
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
            element_types
                .iter()
                .map(|element_type| element_type.name())
                .collect::<HashSet<_>>()
                .difference(
                    &update
                        .elements()
                        .iter()
                        .map(|element| element.name())
                        .collect(),
                )
                .map(|&name| {
                    RecordElement::new(
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
                .chain(update.elements().iter().cloned())
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
    use position::test::PositionFake; use position::Position;
    use hir::types;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_record_update() {
        let record_type = types::Record::new("r", Position::fake());

        assert_eq!(
            transform(
                &RecordUpdate::new(
                    record_type.clone(),
                    Variable::new("r", Position::fake()),
                    vec![RecordElement::new(
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
                            types::RecordElement::new("x", types::Number::new(Position::fake())),
                            types::RecordElement::new("y", types::None::new(Position::fake()))
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
                        RecordElement::new(
                            "x",
                            RecordDeconstruction::new(
                                Some(record_type.into()),
                                Variable::new(RECORD_NAME, Position::fake()),
                                "x",
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        RecordElement::new("y", None::new(Position::fake()), Position::fake())
                    ],
                    Position::fake()
                ),
                Position::fake()
            )
            .into())
        );
    }
}
