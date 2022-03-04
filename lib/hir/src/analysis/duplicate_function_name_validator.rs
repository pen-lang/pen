use super::AnalysisError;
use crate::ir::*;
use fnv::FnvHashMap;
use position::Position;

pub fn validate(module: &Module) -> Result<(), AnalysisError> {
    let mut definitions = FnvHashMap::<&str, &Position>::default();

    for definition in module.definitions() {
        if let Some(&position) = definitions.get(definition.name()) {
            return Err(AnalysisError::DuplicateFunctionNames(
                position.clone(),
                definition.position().clone(),
            ));
        }

        definitions.insert(definition.name(), definition.position());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{DefinitionFake, ModuleFake},
        types,
    };
    use position::{test::PositionFake, Position};

    #[test]
    fn validate_module() {
        let definition = Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            validate(&Module::empty().set_definitions(vec![definition.clone(), definition])),
            Err(AnalysisError::DuplicateFunctionNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }
}
