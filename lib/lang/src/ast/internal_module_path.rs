#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InternalModulePath {
    components: Vec<String>,
}

impl InternalModulePath {
    pub fn new(components: Vec<String>) -> Self {
        Self { components }
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }
}
