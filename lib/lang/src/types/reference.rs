use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Reference {
    name: String,
    source_information: SourceInformation,
}

impl Reference {
    pub fn new(name: impl Into<String>, source_information: SourceInformation) -> Self {
        Self {
            name: name.into(),
            source_information,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_information(&self) -> &SourceInformation {
        &self.source_information
    }
}
