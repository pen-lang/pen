use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PackageConfiguration {
    pub dependencies: HashMap<String, url::Url>,
}
