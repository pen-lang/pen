use super::{type_canonicalizer, TypeError};
use crate::types::*;
use fnv::{FnvHashMap, FnvHashSet};
use position::Position;

pub fn resolve<'a>(
    type_: &Type,
    position: &Position,
    types: &FnvHashMap<String, Type>,
    records: &'a FnvHashMap<String, Vec<RecordField>>,
) -> Result<&'a [RecordField], TypeError> {
    let record = type_canonicalizer::canonicalize_record(type_, types)?
        .ok_or_else(|| TypeError::RecordExpected(position.clone()))?;

    Ok(records
        .get(record.name())
        .ok_or_else(|| TypeError::RecordNotFound(record.clone()))?)
}
