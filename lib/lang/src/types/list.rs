use super::Type;
use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct List {
    element: Arc<Type>,
    source_information: SourceInformation,
}

impl List {
    pub fn new(element: impl Into<Type>, source_information: SourceInformation) -> Self {
        Self {
            element: Arc::new(element.into()),
            source_information,
        }
    }

    pub fn element(&self) -> &Type {
        &self.element
    }

    pub fn source_information(&self) -> &SourceInformation {
        &self.source_information
    }
}
