use super::module_path::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub struct Export {
    module_path: ModulePath,
    names: Vec<String>,
}

impl Export {
    pub fn new(module_path: impl Into<ModulePath>, names: Vec<String>) -> Self {
        Self {
            module_path: module_path.into(),
            names,
        }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }
}
