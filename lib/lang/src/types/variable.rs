use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Variable {
    id: usize,
    source_information: SourceInformation,
}

impl Variable {
    pub fn new(id: usize, source_information: SourceInformation) -> Self {
        Self {
            id,
            source_information,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn source_information(&self) -> &SourceInformation {
        &self.source_information
    }
}
