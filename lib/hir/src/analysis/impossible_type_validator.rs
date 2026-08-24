use super::{AnalysisContext, record_field_resolver, type_resolver};
use crate::{analysis::AnalysisError, ir::*, types::Type};
use fnv::FnvHashSet;
use std::convert::identity;

pub fn validate(context: &AnalysisContext, module: &Module) -> Result<(), AnalysisError> {
    for definition in module.type_definitions() {
        validate_type_definition(context, definition)?;
    }

    Ok(())
}

fn validate_type_definition(
    context: &AnalysisContext,
    definition: &TypeDefinition,
) -> Result<(), AnalysisError> {
    if are_any_type_recursive(
        context,
        definition.name(),
        &FnvHashSet::default(),
        definition.fields().iter().map(|field| field.type_()),
    )? {
        Err(AnalysisError::ImpossibleRecord(
            definition.position().clone(),
        ))
    } else {
        Ok(())
    }
}

fn is_type_recursive(
    context: &AnalysisContext,
    name: &str,
    cache: &FnvHashSet<&str>,
    type_: &Type,
) -> Result<bool, AnalysisError> {
    Ok(match type_ {
        Type::Reference(reference) => is_type_recursive(
            context,
            name,
            cache,
            &type_resolver::resolve(reference, context.types())?,
        )?,
        Type::Record(record) => {
            !cache.contains(record.name())
                && (name == record.name()
                    || are_any_type_recursive(
                        context,
                        name,
                        &cache.clone().into_iter().chain([record.name()]).collect(),
                        record_field_resolver::resolve_record(record, context.records())?
                            .iter()
                            .map(|field| field.type_()),
                    )?)
        }
        Type::Union(union) => [union.lhs(), union.rhs()]
            .into_iter()
            .map(|type_| is_type_recursive(context, name, cache, type_))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(identity),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Error(_)
        | Type::Function(_)
        | Type::List(_)
        | Type::Map(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => false,
    })
}

fn are_any_type_recursive<'a>(
    context: &AnalysisContext,
    name: &str,
    cache: &FnvHashSet<&str>,
    types: impl IntoIterator<Item = &'a Type>,
) -> Result<bool, AnalysisError> {
    Ok(types
        .into_iter()
        .map(|type_| is_type_recursive(context, name, cache, type_))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(identity))
}

#[cfg(test)]
mod tests {
    use super::{super::type_collector, *};
    use crate::{
        test::{ModuleFake, RecordFake, TypeAliasFake, TypeDefinitionFake},
        types,
    };
    use position::{Position, test::PositionFake};

    fn validate_module(module: &Module) -> Result<(), AnalysisError> {
        validate(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_record_fields(module),
            ),
            module,
        )
    }

    #[test]
    fn validate_record() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![],
            false,
            false,
            false,
        )]);

        assert!(matches!(validate_module(&module), Ok(())));
    }

    #[test]
    fn validate_record_with_field() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![types::RecordField::new(
                "x",
                types::None::new(Position::fake()),
            )],
            false,
            false,
            false,
        )]);

        assert!(matches!(validate_module(&module), Ok(())));
    }

    #[test]
    fn validate_recursive_record_with_one_field() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![types::RecordField::new("x", types::Record::fake("a"))],
            false,
            false,
            false,
        )]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }

    #[test]
    fn validate_recursive_record_with_two_fields() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![
                types::RecordField::new("x", types::Record::fake("a")),
                types::RecordField::new("y", types::Record::fake("a")),
            ],
            false,
            false,
            false,
        )]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }

    #[test]
    fn validate_recursive_record_with_two_fields_of_none_and_record() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![
                types::RecordField::new("x", types::None::new(Position::fake())),
                types::RecordField::new("y", types::Record::fake("a")),
            ],
            false,
            false,
            false,
        )]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }

    #[test]
    fn validate_recursive_record_with_union() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![types::RecordField::new(
                "x",
                types::Union::new(
                    types::Record::fake("a"),
                    types::None::new(Position::fake()),
                    Position::fake(),
                ),
            )],
            false,
            false,
            false,
        )]);

        assert!(matches!(validate_module(&module), Ok(())));
    }

    #[test]
    fn validate_recursive_record_with_reference() {
        let module = Module::empty()
            .set_type_definitions(vec![TypeDefinition::fake(
                "a",
                vec![types::RecordField::new(
                    "x",
                    types::Reference::new("b", Position::fake()),
                )],
                false,
                false,
                false,
            )])
            .set_type_aliases(vec![TypeAlias::fake(
                "b",
                types::Reference::new("a", Position::fake()),
                false,
                false,
            )]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }

    #[test]
    fn validate_recursive_record_with_reference_to_union() {
        let module = Module::empty()
            .set_type_definitions(vec![TypeDefinition::fake(
                "a",
                vec![types::RecordField::new(
                    "x",
                    types::Reference::new("b", Position::fake()),
                )],
                false,
                false,
                false,
            )])
            .set_type_aliases(vec![TypeAlias::fake(
                "b",
                types::Union::new(
                    types::Record::fake("a"),
                    types::None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
                false,
            )]);

        assert!(matches!(validate_module(&module), Ok(())));
    }

    #[test]
    fn validate_mutually_recursive_records() {
        let module = Module::empty().set_type_definitions(vec![
            TypeDefinition::fake(
                "a",
                vec![types::RecordField::new("x", types::Record::fake("b"))],
                false,
                false,
                false,
            ),
            TypeDefinition::fake(
                "b",
                vec![types::RecordField::new("x", types::Record::fake("a"))],
                false,
                false,
                false,
            ),
        ]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }

    #[test]
    fn validate_mutually_recursive_records_with_union() {
        let module = Module::empty().set_type_definitions(vec![
            TypeDefinition::fake(
                "a",
                vec![types::RecordField::new(
                    "x",
                    types::Union::new(
                        types::Record::fake("b"),
                        types::None::new(Position::fake()),
                        Position::fake(),
                    ),
                )],
                false,
                false,
                false,
            ),
            TypeDefinition::fake(
                "b",
                vec![types::RecordField::new("x", types::Record::fake("a"))],
                false,
                false,
                false,
            ),
        ]);

        assert!(matches!(validate_module(&module), Ok(())));
    }

    #[test]
    fn validate_record_with_recursive_record() {
        let module = Module::empty().set_type_definitions(vec![
            TypeDefinition::fake(
                "a",
                vec![types::RecordField::new("x", types::Record::fake("b"))],
                false,
                false,
                false,
            ),
            TypeDefinition::fake(
                "b",
                vec![types::RecordField::new("x", types::Record::fake("b"))],
                false,
                false,
                false,
            ),
        ]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }
}
