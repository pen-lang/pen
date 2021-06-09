#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Record {
    name: String,
}

impl Record {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
