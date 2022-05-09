use super::DropVariables;
use fnv::FnvHashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct RetainVariables {
    variables: FnvHashMap<String, String>,
    drop: DropVariables,
}

impl RetainVariables {
    pub fn new(variables: FnvHashMap<String, String>, drop: DropVariables) -> Self {
        Self { variables, drop }
    }

    pub fn variables(&self) -> &FnvHashMap<String, String> {
        &self.variables
    }

    pub fn drop(&self) -> &DropVariables {
        &self.drop
    }
}
