
use super::{external_module_path::ExternalModulePath, internal_module_path::InternalModulePath};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
