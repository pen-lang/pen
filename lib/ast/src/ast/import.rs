use super::module_path::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    module_path: ModulePath,
    prefix: Option<String>,
}

impl Import {
    pub fn new(module_path: impl Into<ModulePath>, prefix: Option<String>) -> Self {
        Self {
            module_path: module_path.into(),
            prefix,
        }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}
