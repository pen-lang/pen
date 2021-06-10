use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Any {
    source_information: SourceInformation,
}

impl Any {
    pub fn new(source_information: SourceInformation) -> Self {
        Self { source_information }
    }

    pub fn source_information(&self) -> &SourceInformation {
        &self.source_information
    }
}
