use crate::IDENTIFIER_SEPARATOR;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl Display for ExternalModulePath {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}{}",
            &self.package,
            IDENTIFIER_SEPARATOR,
            self.components.join(IDENTIFIER_SEPARATOR),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(
            ExternalModulePath::new("foo", vec!["bar".into()]).to_string(),
            "foo'bar"
        );
    }
}
