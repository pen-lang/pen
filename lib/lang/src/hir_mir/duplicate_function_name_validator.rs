use super::CompileError;
use crate::hir::*;
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
    use crate::types;
    use position::Position;

    #[test]
    fn validate_module() {
        let definition = Definition::without_source(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::dummy()),
                None::new(Position::dummy()),
                Position::dummy(),
            ),
            false,
        );

        assert_eq!(
            validate(&Module::empty().set_definitions(vec![definition.clone(), definition])),
            Err(CompileError::DuplicateFunctionNames(
                Position::dummy(),
                Position::dummy()
            ))
        );
    }
}
