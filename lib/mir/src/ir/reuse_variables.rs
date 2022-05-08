use super::DropVariables;
use fnv::FnvHashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ReuseVariables {
    variables: FnvHashMap<String, String>,
    drop: DropVariables,
}

impl ReuseVariables {
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
