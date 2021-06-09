#[derive(Clone, Debug, PartialEq)]
pub struct ForeignDefinition {
    name: String,
    foreign_name: String,
}

impl ForeignDefinition {
    pub fn new(name: impl Into<String>, foreign_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn foreign_name(&self) -> &str {
        &self.foreign_name
    }
}
