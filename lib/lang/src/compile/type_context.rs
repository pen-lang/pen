pub struct TypeContext {
    records: HashMap<String, Type>,
    types: HashMap<String, Type>,
}

impl TypeContext {
    pub fn new(module: &Module) -> Self {
        Self {}
    }

    pub fn records(&self) -> &HashMap<String, Vec<RecordElement>> {
        &self.records
    }

    pub fn types(&self) -> &HashMap<String, Type> {
        &self.types
    }
}
