use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: HashMap<String, String>,
}

impl JsonPackageConfiguration {
    pub fn new(dependencies: &HashMap<String, url::Url>) -> Self {
        Self {
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
        }
    }

    pub fn get_dependencies(&self) -> Result<HashMap<String, url::Url>, url::ParseError> {
        self.dependencies
            .iter()
            .map(|(name, url_string)| Ok((name.clone(), url::Url::parse(url_string)?)))
            .collect::<Result<_, url::ParseError>>()
    }
}
