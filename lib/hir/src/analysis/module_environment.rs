use super::type_extractor;
use crate::{ir::*, types::Type};
use std::collections::HashMap;

// TODO Use FnvHashMap for deterministic build.
pub fn create(module: &Module) -> HashMap<String, Type> {
    module
        .function_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                declaration.type_().clone().into(),
            )
        })
        .chain(
            module
                .foreign_declarations()
                .iter()
                .map(|declaration| (declaration.name().into(), declaration.type_().clone())),
        )
        .chain(module.function_definitions().iter().map(|definition| {
            (
                definition.name().into(),
                type_extractor::extract_from_lambda(definition.lambda()).into(),
            )
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::ModuleFake, types};
    use position::{Position, test::PositionFake};

    #[test]
    fn create_with_foreign_declaration() {
        let type_ =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            create(
                &Module::empty().set_foreign_declarations(vec![ForeignDeclaration::new(
                    "foo",
                    "bar",
                    CallingConvention::Native,
                    type_.clone(),
                    Position::fake()
                )])
            ),
            [("foo".into(), type_.into())].into_iter().collect()
        );
    }
}
