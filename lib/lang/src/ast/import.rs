use super::module_path::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    module_path: ModulePath,
}

impl Import {
    pub fn new(module_path: ModulePath) -> Self {
        Self { module_path }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }
}
