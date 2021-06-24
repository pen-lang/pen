use super::{type_context::TypeContext, CompileError};
use crate::{
    position::Position,
    types::{analysis::type_resolver, Type},
};
use std::collections::HashMap;

pub fn resolve_elements<'a>(
    type_: &Type,
    position: &Position,
    type_context: &'a TypeContext,
) -> Result<&'a HashMap<String, Type>, CompileError> {
    type_resolver::resolve_record_elements(type_, type_context.types(), type_context.records())?
        .ok_or_else(|| CompileError::RecordExpected(position.clone()))
}
