use super::{external_module_path::ExternalModulePath, internal_module_path::InternalModulePath};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ModulePath {
    External(ExternalModulePath),
    Internal(InternalModulePath),
}

impl From<ExternalModulePath> for ModulePath {
    fn from(path: ExternalModulePath) -> Self {
        Self::External(path)
    }
}

impl From<InternalModulePath> for ModulePath {
    fn from(path: InternalModulePath) -> Self {
        Self::Internal(path)
    }
}

impl Display for ModulePath {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::External(path) => write!(formatter, "{path}"),
            Self::Internal(path) => write!(formatter, "{path}"),
        }
    }
}
