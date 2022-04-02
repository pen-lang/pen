use super::module_path::ModulePath;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    module_path: ModulePath,
    prefix: Option<String>,
    unqualified_names: Vec<String>,
    position: Position,
}

impl Import {
    pub fn new(
        module_path: impl Into<ModulePath>,
        prefix: Option<String>,
        unqualified_names: Vec<String>,
        position: Position,
    ) -> Self {
        Self {
            module_path: module_path.into(),
            prefix,
            unqualified_names,
            position,
        }
    }

    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn unqualified_names(&self) -> &[String] {
        &self.unqualified_names
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
