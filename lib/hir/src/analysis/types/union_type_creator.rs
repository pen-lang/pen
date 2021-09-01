use crate::types::{self, Type};
use position::Position;

pub fn create(types: &[Type], position: &Position) -> Option<Type> {
    types
        .iter()
        .cloned()
        .reduce(|left, right| types::Union::new(left, right, position.clone()).into())
}
