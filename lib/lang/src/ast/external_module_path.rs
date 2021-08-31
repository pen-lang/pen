
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ExternalModulePath {
    package: String,
    components: Vec<String>,
}

impl ExternalModulePath {
    pub fn new(package: impl Into<String>, components: Vec<String>) -> Self {
        Self {
            package: package.into(),
            components,
        }
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }
}
