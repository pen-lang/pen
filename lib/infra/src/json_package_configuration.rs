use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: BTreeMap<String, String>,
    pub system: Option<bool>,
}

impl JsonPackageConfiguration {
    pub fn new(dependencies: BTreeMap<String, url::Url>, system: bool) -> Self {
        Self {
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
            system: if system { Some(true) } else { None },
        }
    }

    pub fn into_configuration(
        &self,
        base_url: &url::Url,
    ) -> Result<app::PackageConfiguration, url::ParseError> {
        Ok(app::PackageConfiguration::new(
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
                .collect::<Result<_, url::ParseError>>()?,
            self.system.unwrap_or_default(),
        ))
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
