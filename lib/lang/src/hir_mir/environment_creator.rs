use super::type_extractor;
use crate::{hir::*, types::Type};
use std::collections::HashMap;

pub fn create_from_module(module: &Module) -> HashMap<String, Type> {
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
    use crate::{test, types};

    #[test]
    fn create_with_foreign_declaration() {
        let type_ =
            types::Function::new(vec![], types::None::new(test::position()), test::position());

        assert_eq!(
            create_from_module(&Module::empty().set_foreign_declarations(vec![
                ForeignDeclaration::new(
                    "foo",
                    "bar",
                    CallingConvention::Native,
                    type_.clone(),
                    test::position()
                )
            ])),
            vec![("foo".into(), type_.into())].into_iter().collect()
        );
    }
}
