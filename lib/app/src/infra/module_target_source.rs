pub struct ModuleTargetSource {
    package_name: String,
    module_name: String,
}

impl ModuleTargetSource {
    pub fn new(package_name: impl Into<String>, module_name: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            module_name: module_name.into(),
        }
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
