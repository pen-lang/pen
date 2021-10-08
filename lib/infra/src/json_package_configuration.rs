use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: HashMap<String, String>,
}

impl JsonPackageConfiguration {
    pub fn new(dependencies: HashMap<String, url::Url>) -> Self {
        Self {
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
        }
    }

    pub fn dependencies(
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

#[cfg(test)]
mod tests {
    #[test]
    fn parse_relative_path() {
        assert_eq!(
            url::Url::options()
                .base_url(Some(&url::Url::parse("file:///foo/bar/").unwrap()))
                .parse("../baz"),
            url::Url::parse("file:///foo/baz")
        );
    }
}
