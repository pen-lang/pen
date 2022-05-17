use super::DropVariables;
use fnv::FnvHashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct RetainHeap {
    // a map from variable name to reuse ID
    ids: FnvHashMap<String, String>,
    drop: DropVariables,
}

impl RetainHeap {
    pub fn new(ids: FnvHashMap<String, String>, drop: DropVariables) -> Self {
        Self { ids, drop }
    }

    pub fn ids(&self) -> &FnvHashMap<String, String> {
        &self.ids
    }

    pub fn drop(&self) -> &DropVariables {
        &self.drop
    }
}
