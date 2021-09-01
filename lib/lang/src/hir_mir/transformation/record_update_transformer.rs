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
    use crate::{test, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_record_update() {
        let record_type = types::Record::new("r", test::position());

        assert_eq!(
            transform(
                &RecordUpdate::new(
                    record_type.clone(),
                    Variable::new("r", test::position()),
                    vec![RecordElement::new(
                        "y",
                        None::new(test::position()),
                        test::position()
                    )],
                    test::position()
                ),
                &TypeContext::dummy(
                    Default::default(),
                    vec![(
                        "r".into(),
                        vec![
                            types::RecordElement::new("x", types::Number::new(test::position())),
                            types::RecordElement::new("y", types::None::new(test::position()))
                        ]
                    )]
                    .into_iter()
                    .collect()
                )
            ),
            Ok(Let::new(
                Some(RECORD_NAME.into()),
                Some(record_type.clone().into()),
                Variable::new("r", test::position()),
                RecordConstruction::new(
                    record_type.clone(),
                    vec![
                        RecordElement::new(
                            "x",
                            RecordDeconstruction::new(
                                Some(record_type.into()),
                                Variable::new(RECORD_NAME, test::position()),
                                "x",
                                test::position()
                            ),
                            test::position()
                        ),
                        RecordElement::new("y", None::new(test::position()), test::position())
                    ],
                    test::position()
                ),
                test::position()
            )
            .into())
        );
    }
}
