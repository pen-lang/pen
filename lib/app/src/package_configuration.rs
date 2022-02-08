use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageConfiguration {
    dependencies: BTreeMap<String, url::Url>,
    system: bool,
}

impl PackageConfiguration {
    pub fn new(dependencies: BTreeMap<String, url::Url>, system: bool) -> Self {
        Self {
            dependencies,
            system,
        }
    }

    pub fn dependencies(&self) -> &BTreeMap<String, url::Url> {
        &self.dependencies
    }

    pub fn is_system(&self) -> bool {
        self.system
    }
}
