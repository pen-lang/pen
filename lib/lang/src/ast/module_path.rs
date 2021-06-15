use super::{external_module_path::ExternalModulePath, internal_module_path::InternalModulePath};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
