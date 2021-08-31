
use super::super::error::CompileError;
use crate::{hir::*, hir_mir::type_context::TypeContext, types::analysis::record_element_resolver};
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
    use crate::types;
    use position::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_record_update() {
        let record_type = types::Record::new("r", Position::dummy());

        assert_eq!(
            transform(
                &RecordUpdate::new(
                    record_type.clone(),
                    Variable::new("r", Position::dummy()),
                    vec![RecordElement::new(
                        "y",
                        None::new(Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                ),
                &TypeContext::dummy(
                    Default::default(),
                    vec![(
                        "r".into(),
                        vec![
                            types::RecordElement::new("x", types::Number::new(Position::dummy())),
                            types::RecordElement::new("y", types::None::new(Position::dummy()))
                        ]
                    )]
                    .into_iter()
                    .collect()
                )
            ),
            Ok(Let::new(
                Some(RECORD_NAME.into()),
                Some(record_type.clone().into()),
                Variable::new("r", Position::dummy()),
                RecordConstruction::new(
                    record_type.clone(),
                    vec![
                        RecordElement::new(
                            "x",
                            RecordDeconstruction::new(
                                Some(record_type.into()),
                                Variable::new(RECORD_NAME, Position::dummy()),
                                "x",
                                Position::dummy()
                            ),
                            Position::dummy()
                        ),
                        RecordElement::new("y", None::new(Position::dummy()), Position::dummy())
                    ],
                    Position::dummy()
                ),
                Position::dummy()
            )
            .into())
        );
    }
}
