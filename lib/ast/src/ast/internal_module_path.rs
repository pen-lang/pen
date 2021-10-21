use crate::IDENTIFIER_SEPARATOR;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl Display for InternalModulePath {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}{}",
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
            InternalModulePath::new(vec!["foo".into(), "bar".into()]).to_string(),
            "'foo'bar"
        );
    }
}
