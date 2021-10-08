use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: HashMap<String, String>,
}

impl JsonPackageConfiguration {
    pub fn from_dependency_urls(dependencies: HashMap<String, url::Url>) -> Self {
        Self {
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
        }
    }

    pub fn into_dependency_urls(
        &self,
        base_url: &url::Url,
    ) -> Result<HashMap<String, url::Url>, url::ParseError> {
        self.dependencies
            .iter()
            .map(|(name, url_string)| {
                Ok((
                    name.clone(),
                    url::Url::options()
                        .base_url(Some(base_url))
                        .parse(url_string)?,
                ))
            })
            .collect::<Result<_, url::ParseError>>()
    }
}
