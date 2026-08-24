use super::AnalysisError;
use crate::ir::*;
use fnv::FnvHashMap;
use position::Position;

pub fn validate(module: &Module) -> Result<(), AnalysisError> {
    let mut definitions = FnvHashMap::<&str, &Position>::default();

    for (name, position) in module
        .type_definitions()
        .iter()
        .map(|definition| (definition.name(), definition.position()))
        .chain(
            module
                .type_aliases()
                .iter()
                .map(|alias| (alias.name(), alias.position())),
        )
    {
        if let Some(&position) = definitions.get(name) {
            return Err(AnalysisError::DuplicateTypeNames(
                position.clone(),
                position.clone(),
            ));
        }

        definitions.insert(name, position);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{ModuleFake, TypeAliasFake, TypeDefinitionFake},
        types,
    };
    use position::{Position, test::PositionFake};

    #[test]
    fn validate_type_definitions() {
        let definition = TypeDefinition::fake("x", vec![], false, false, false);

        assert_eq!(
            validate(&Module::empty().set_type_definitions(vec![definition.clone(), definition])),
            Err(AnalysisError::DuplicateTypeNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }

    #[test]
    fn validate_type_aliases() {
        let alias = TypeAlias::fake("x", types::None::new(Position::fake()), false, false);

        assert_eq!(
            validate(&Module::empty().set_type_aliases(vec![alias.clone(), alias])),
            Err(AnalysisError::DuplicateTypeNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }

    #[test]
    fn validate_type_definition_and_alias() {
        assert_eq!(
            validate(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "x",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_type_aliases(vec![TypeAlias::fake(
                        "x",
                        types::None::new(Position::fake()),
                        false,
                        false
                    )])
            ),
            Err(AnalysisError::DuplicateTypeNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }
}
