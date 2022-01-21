use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;

pub fn collect(module: &Module) -> FnvHashMap<String, Type> {
    module
        .type_definitions()
        .iter()
        .map(|definition| {
            (
                definition.name().into(),
                types::Record::new(definition.name(), definition.position().clone()).into(),
            )
        })
        .chain(
            module
                .type_aliases()
                .iter()
                .map(|alias| (alias.name().into(), alias.type_().clone())),
        )
        .collect()
}
