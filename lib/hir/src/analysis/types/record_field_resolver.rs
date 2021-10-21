use super::{type_canonicalizer, TypeError};
use crate::types::*;
use position::Position;
use std::collections::BTreeMap;

pub fn resolve<'a>(
    type_: &Type,
    position: &Position,
    types: &BTreeMap<String, Type>,
    records: &'a BTreeMap<String, Vec<RecordField>>,
) -> Result<&'a [RecordField], TypeError> {
    let record = type_canonicalizer::canonicalize_record(type_, types)?
        .ok_or_else(|| TypeError::RecordExpected(position.clone()))?;

    Ok(records
        .get(record.name())
        .ok_or_else(|| TypeError::RecordNotFound(record.clone()))?)
}
