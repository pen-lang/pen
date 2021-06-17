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
        .chain(module.definitions().iter().map(|definition| {
            (
                definition.name().into(),
                type_extractor::extract_from_lambda(definition.lambda()).into(),
            )
        }))
        .collect()
}
