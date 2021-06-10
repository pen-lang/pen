use super::Type;
use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Union {
    lhs: Arc<Type>,
    rhs: Arc<Type>,
    source_information: SourceInformation,
}

impl Union {
    pub fn new(
        lhs: impl Into<Type>,
        rhs: impl Into<Type>,
        source_information: SourceInformation,
    ) -> Self {
        Self {
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
            source_information,
        }
    }

    pub fn lhs(&self) -> &Type {
        &self.lhs
    }

    pub fn rhs(&self) -> &Type {
        &self.rhs
    }

    pub fn source_information(&self) -> &SourceInformation {
        &self.source_information
    }
}
