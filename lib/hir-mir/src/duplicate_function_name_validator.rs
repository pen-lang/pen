use super::CompileError;
use hir::ir::*;
use position::Position;
use std::collections::HashMap;

pub fn validate(module: &Module) -> Result<(), CompileError> {
    let mut definitions = HashMap::<&str, &Position>::new();

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
    use position::test::PositionFake; use position::Position;
    use hir::{
        test::{DefinitionFake, ModuleFake},
        types,
    };

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
