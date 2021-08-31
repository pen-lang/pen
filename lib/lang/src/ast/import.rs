
use super::module_path::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    module_path: ModulePath,
}

impl Import {
    pub fn new(module_path: impl Into<ModulePath>) -> Self {
        Self {
            module_path: module_path.into(),
        }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }
}
