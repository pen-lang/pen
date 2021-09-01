use super::{type_canonicalizer, TypeError};
use crate::types::*;
use position::Position;
use std::collections::HashMap;

pub fn resolve<'a>(
    type_: &Type,
    position: &Position,
    types: &HashMap<String, Type>,
    records: &'a HashMap<String, Vec<RecordElement>>,
) -> Result<&'a [RecordElement], TypeError> {
    let record = type_canonicalizer::canonicalize_record(type_, types)?
        .ok_or_else(|| TypeError::RecordExpected(position.clone()))?;

    Ok(records
        .get(record.name())
        .ok_or_else(|| TypeError::RecordNotFound(record.clone()))?)
}
