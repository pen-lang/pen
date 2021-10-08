use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: HashMap<String, String>,
}

impl JsonPackageConfiguration {
    pub fn from_dependency_urls(dependencies: &HashMap<String, url::Url>) -> Self {
        Self {
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
        }
    }
}
