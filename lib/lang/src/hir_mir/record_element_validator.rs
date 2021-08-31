use super::{type_context::TypeContext, CompileError};
use crate::{
    hir::{analysis::expression_visitor, *},
    types::analysis::type_canonicalizer,
};
use std::collections::HashSet;

pub fn validate(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    let open_records = collect_open_records(module.type_definitions());

    for expression in collect_expressions(module) {
        match expression {
            Expression::RecordConstruction(construction) => {
                let record_type = type_canonicalizer::canonicalize_record(
                    construction.type_(),
                    type_context.types(),
                )?
                .ok_or_else(|| CompileError::RecordExpected(construction.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordElementPrivate(
                        construction.position().clone(),
                    ));
                }
            }
            Expression::RecordDeconstruction(deconstruction) => {
                let record_type = type_canonicalizer::canonicalize_record(
                    deconstruction.type_().ok_or_else(|| {
                        CompileError::TypeNotInferred(deconstruction.position().clone())
                    })?,
                    type_context.types(),
                )?
                .ok_or_else(|| CompileError::RecordExpected(deconstruction.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordElementPrivate(
                        deconstruction.position().clone(),
                    ));
                }
            }
            Expression::RecordUpdate(update) => {
                let record_type =
                    type_canonicalizer::canonicalize_record(update.type_(), type_context.types())?
                        .ok_or_else(|| CompileError::RecordExpected(update.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordElementPrivate(
                        update.position().clone(),
                    ));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn collect_expressions(module: &Module) -> Vec<Expression> {
    let mut expressions = vec![];

    expression_visitor::visit(module, |expression: &Expression| match expression {
        Expression::RecordConstruction(_)
        | Expression::RecordDeconstruction(_)
        | Expression::RecordUpdate(_) => expressions.push(expression.clone()),
        _ => {}
    });

    expressions
}

fn collect_open_records(type_definitions: &[TypeDefinition]) -> HashSet<String> {
    type_definitions
        .iter()
        .filter_map(|definition| {
            if is_record_open(definition) {
                Some(definition.name().into())
            } else {
                None
            }
        })
        .collect()
}

fn is_record_open(definition: &TypeDefinition) -> bool {
    definition.is_open() || !definition.is_external()
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
        *,
    };
    use crate::types;
    use position::Position;

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            module,
            &TypeContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
            ),
        )
    }

    #[test]
    fn validate_record_construction() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_construction() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_record_deconstruction() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordDeconstruction::new(
                            Some(record_type.into()),
                            Variable::new("x", Position::dummy()),
                            "x",
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_deconstruction() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordDeconstruction::new(
                            Some(record_type.into()),
                            Variable::new("x", Position::dummy()),
                            "x",
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_record_update() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("x", Position::dummy()),
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_update() {
        let record_type = types::Record::new("r", Position::dummy());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("x", Position::dummy()),
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }
}
