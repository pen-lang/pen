use crate::{
    position::Position,
    types::{self, Type},
};

pub fn create_union_type(types: &[Type], position: &Position) -> Option<Type> {
    types
        .iter()
        .cloned()
        .reduce(|left, right| types::Union::new(left, right, position.clone()).into())
}
