use super::{record_field_resolver, type_resolver, AnalysisContext};
use crate::{analysis::AnalysisError, ir::*, types::Type};
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
    type_: &Type,
) -> Result<bool, AnalysisError> {
    Ok(match type_ {
        Type::Reference(reference) => is_type_recursive(
            context,
            name,
            &type_resolver::resolve(reference, context.types())?,
        )?,
        Type::Record(record) => {
            name == record.name()
                || are_any_type_recursive(
                    context,
                    name,
                    record_field_resolver::resolve_record(record, context.records())?
                        .iter()
                        .map(|field| field.type_()),
                )?
        }
        Type::Union(union) => [union.lhs(), union.rhs()]
            .into_iter()
            .map(|type_| is_type_recursive(context, name, type_))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(identity),
        Type::Any(_)
        | Type::Boolean(_)
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
    types: impl IntoIterator<Item = &'a Type>,
) -> Result<bool, AnalysisError> {
    Ok(types
        .into_iter()
        .map(|type_| is_type_recursive(context, name, type_))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(identity))
}

#[cfg(test)]
mod tests {
    use super::{super::type_collector, *};
    use crate::{
        test::{ModuleFake, TypeDefinitionFake},
        types,
    };
    use position::{test::PositionFake, Position};

    fn validate_module(module: &Module) -> Result<(), AnalysisError> {
        validate(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_records(module),
                None,
            ),
            &module,
        )
    }

    #[test]
    fn validate_self_recursive_record_with_one_field() {
        let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
            "a",
            vec![types::RecordField::new(
                "x",
                types::Record::new("a", Position::fake()),
            )],
            false,
            false,
            false,
        )]);

        assert!(matches!(
            validate_module(&module),
            Err(AnalysisError::ImpossibleRecord(_))
        ));
    }
}
