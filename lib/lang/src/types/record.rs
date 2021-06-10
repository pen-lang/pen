use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Record {
    name: String,
    source_information: SourceInformation,
}

impl Record {
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
