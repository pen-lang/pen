use super::type_::Type;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordBody {
    fields: Vec<Type>,
}

impl RecordBody {
    pub const fn new(fields: Vec<Type>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[Type] {
        &self.fields
    }
}
