use super::module_path::ModulePath;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    module_path: ModulePath,
    prefix: Option<String>,
    names: Vec<String>,
}

impl Import {
    pub fn new(
        module_path: impl Into<ModulePath>,
        prefix: Option<String>,
        names: Vec<String>,
    ) -> Self {
        Self {
            module_path: module_path.into(),
            prefix,
            names,
        }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }
}
