use super::{compile_context::CompileContext, CompileError};
use hir::{
    analysis::{ir::expression_visitor, types::type_canonicalizer},
    ir::*,
};
use std::collections::BTreeSet;

pub fn validate(module: &Module, compile_context: &CompileContext) -> Result<(), CompileError> {
    let open_records = collect_open_records(module.type_definitions());

    for expression in collect_expressions(module) {
        match expression {
            Expression::RecordConstruction(construction) => {
                let record_type = type_canonicalizer::canonicalize_record(
                    construction.type_(),
                    compile_context.types(),
                )?
                .ok_or_else(|| CompileError::RecordExpected(construction.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordFieldPrivate(
                        construction.position().clone(),
                    ));
                }
            }
            Expression::RecordDeconstruction(deconstruction) => {
                let record_type = type_canonicalizer::canonicalize_record(
                    deconstruction.type_().ok_or_else(|| {
                        CompileError::TypeNotInferred(deconstruction.position().clone())
                    })?,
                    compile_context.types(),
                )?
                .ok_or_else(|| CompileError::RecordExpected(deconstruction.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordFieldPrivate(
                        deconstruction.position().clone(),
                    ));
                }
            }
            Expression::RecordUpdate(update) => {
                let record_type = type_canonicalizer::canonicalize_record(
                    update.type_(),
                    compile_context.types(),
                )?
                .ok_or_else(|| CompileError::RecordExpected(update.position().clone()))?;

                if !open_records.contains(record_type.name()) {
                    return Err(CompileError::RecordFieldPrivate(update.position().clone()));
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

fn collect_open_records(type_definitions: &[TypeDefinition]) -> BTreeSet<String> {
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
    !definition.is_external() || definition.is_public() && definition.is_open()
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            concurrency_configuration::CONCURRENCY_CONFIGURATION,
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
        *,
    };
    use hir::{
        test::{DefinitionFake, ModuleFake, TypeDefinitionFake},
        types,
    };
    use position::{test::PositionFake, Position};

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            module,
            &CompileContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
                &CONCURRENCY_CONFIGURATION,
            ),
        )
    }

    #[test]
    fn validate_record_construction() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_construction() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_record_deconstruction() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            Some(record_type.into()),
                            Variable::new("x", Position::fake()),
                            "x",
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_deconstruction() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            Some(record_type.into()),
                            Variable::new("x", Position::fake()),
                            "x",
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_record_update() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::fake()),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("x", Position::fake()),
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_update() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", record_type.clone())],
                        types::None::new(Position::fake()),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("x", Position::fake()),
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_record_construction_with_external_private_record() {
        let record_type = types::Record::new("r", Position::fake());

        validate_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    true,
                    false,
                    true,
                )])
                .set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }
}
