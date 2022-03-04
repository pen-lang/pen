use fnv::FnvHashMap;
use hir::{analysis::type_extractor, ir::*, types::Type};

pub fn create_from_module(module: &Module) -> FnvHashMap<String, Type> {
    module
        .declarations()
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
        .chain(module.definitions().iter().map(|definition| {
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
    use hir::{test::ModuleFake, types};
    use position::{test::PositionFake, Position};

    #[test]
    fn create_with_foreign_declaration() {
        let type_ =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            create_from_module(&Module::empty().set_foreign_declarations(vec![
                ForeignDeclaration::new(
                    "foo",
                    "bar",
                    CallingConvention::Native,
                    type_.clone(),
                    Position::fake()
                )
            ])),
            [("foo".into(), type_.into())].into_iter().collect()
        );
    }
}
