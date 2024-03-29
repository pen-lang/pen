use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;

pub fn collect(module: &Module) -> FnvHashMap<String, Type> {
    collect_records(module)
        .into_iter()
        .chain(
            module
                .type_aliases()
                .iter()
                .map(|alias| (alias.name().into(), alias.type_().clone())),
        )
        .collect()
}

pub fn collect_records(module: &Module) -> FnvHashMap<String, Type> {
    module
        .type_definitions()
        .iter()
        .map(|definition| {
            (
                definition.name().into(),
                types::Record::new(
                    definition.name(),
                    definition.original_name(),
                    definition.position().clone(),
                )
                .into(),
            )
        })
        .collect()
}

pub fn collect_record_fields(module: &Module) -> FnvHashMap<String, Vec<types::RecordField>> {
    module
        .type_definitions()
        .iter()
        .map(|definition| (definition.name().into(), definition.fields().to_vec()))
        .collect()
}
