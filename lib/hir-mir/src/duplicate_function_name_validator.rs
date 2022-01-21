use super::CompileError;
use fnv::{FnvHashMap, FnvHashSet};
use hir::ir::*;
use position::Position;

pub fn validate(module: &Module) -> Result<(), CompileError> {
    let mut definitions = FnvHashMap::<&str, &Position>::default();

    for definition in module.definitions() {
        if let Some(&position) = definitions.get(definition.name()) {
            return Err(CompileError::DuplicateFunctionNames(
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
    use hir::{
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
            Err(CompileError::DuplicateFunctionNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }
}
