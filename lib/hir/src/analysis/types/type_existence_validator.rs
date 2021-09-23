use super::error::TypeError;
use crate::analysis::ir::type_transformer;
use crate::ir::*;
use crate::types::Type;
use std::cell::RefCell;
use std::collections::HashSet;

pub fn validate(
    module: &Module,
    types: &HashSet<String>,
    records: &HashSet<String>,
) -> Result<(), TypeError> {
    for type_ in &collect_types(module) {
        match type_ {
            Type::Record(record) => {
                if !records.contains(record.name()) {
                    return Err(TypeError::RecordNotFound(record.clone()));
                }
            }
            Type::Reference(reference) => {
                if !types.contains(reference.name()) {
                    return Err(TypeError::TypeNotFound(reference.clone()));
                }
            }
            Type::Any(_)
            | Type::Boolean(_)
            | Type::Function(_)
            | Type::List(_)
            | Type::None(_)
            | Type::Number(_)
            | Type::String(_)
            | Type::Union(_) => {}
        }
    }

    Ok(())
}

fn collect_types(module: &Module) -> Vec<Type> {
    let types = RefCell::new(vec![]);

    type_transformer::transform(module, |type_| {
        types.borrow_mut().push(type_.clone());

        type_.clone()
    });

    types.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::ModuleFake;
    use crate::test::TypeAliasFake;
    use crate::test::TypeDefinitionFake;
    use crate::types;
    use position::test::PositionFake;
    use position::Position;

    #[test]
    fn fail_to_validate_non_existent_reference_type_in_type_alias() {
        assert_eq!(
            validate(
                &Module::empty().set_type_aliases(vec![TypeAlias::fake(
                    "x",
                    types::Reference::new("foo", Position::fake()),
                    false,
                    false
                )]),
                &Default::default(),
                &Default::default(),
            ),
            Err(TypeError::TypeNotFound(types::Reference::new(
                "foo",
                Position::fake()
            )))
        );
    }

    #[test]
    fn fail_to_validate_non_existent_reference_type_in_type_definition() {
        assert_eq!(
            validate(
                &Module::empty().set_type_aliases(vec![TypeAlias::fake(
                    "x",
                    types::Reference::new("foo", Position::fake()),
                    false,
                    false
                )]),
                &Default::default(),
                &Default::default(),
            ),
            Err(TypeError::TypeNotFound(types::Reference::new(
                "foo",
                Position::fake()
            )))
        );
    }

    #[test]
    fn fail_to_validate_non_existent_record_type() {
        assert_eq!(
            validate(
                &Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                    "x",
                    vec![types::RecordElement::new(
                        "x",
                        types::Record::new("foo", Position::fake())
                    )],
                    false,
                    false,
                    false
                )]),
                &Default::default(),
                &Default::default(),
            ),
            Err(TypeError::RecordNotFound(types::Record::new(
                "foo",
                Position::fake()
            )))
        );
    }
}
