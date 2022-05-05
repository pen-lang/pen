pub struct ModuleTargetSource {
    package_name: Option<String>,
    module_name: String,
}

impl ModuleTargetSource {
    pub fn new(package_name: Option<String>, module_name: impl Into<String>) -> Self {
        Self {
            package_name,
            module_name: module_name.into(),
        }
    }

    pub fn package_name(&self) -> Option<&str> {
        self.package_name.as_deref()
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
