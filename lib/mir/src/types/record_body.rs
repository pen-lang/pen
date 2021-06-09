use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordBody {
    elements: Vec<Type>,
}

impl RecordBody {
    pub const fn new(elements: Vec<Type>) -> Self {
        RecordBody { elements }
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }
}
