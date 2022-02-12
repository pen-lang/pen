use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonPackageType {
    Application,
    Library,
    System,
}

impl From<app::PackageType> for JsonPackageType {
    fn from(type_: app::PackageType) -> JsonPackageType {
        match type_ {
            app::PackageType::Application => JsonPackageType::Application,
            app::PackageType::Library => JsonPackageType::Library,
            app::PackageType::System => JsonPackageType::System,
        }
    }
}

impl From<JsonPackageType> for app::PackageType {
    fn from(type_: JsonPackageType) -> app::PackageType {
        match type_ {
            JsonPackageType::Application => app::PackageType::Application,
            JsonPackageType::Library => app::PackageType::Library,
            JsonPackageType::System => app::PackageType::System,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub type_: JsonPackageType,
    pub dependencies: BTreeMap<String, String>,
}

impl JsonPackageConfiguration {
    pub fn new(
        type_: impl Into<JsonPackageType>,
        dependencies: BTreeMap<String, url::Url>,
    ) -> Self {
        Self {
            type_: type_.into(),
            dependencies: dependencies
                .iter()
                .map(|(name, url)| (name.clone(), url.as_str().into()))
                .collect(),
        }
    }

    pub fn into_configuration(
        self,
        base_url: &url::Url,
    ) -> Result<app::PackageConfiguration, url::ParseError> {
        Ok(app::PackageConfiguration::new(
            self.type_.into(),
            self.dependencies
                .into_iter()
                .map(|(name, url)| {
                    Ok((
                        name,
                        url::Url::options().base_url(Some(base_url)).parse(&url)?,
                    ))
                })
                .collect::<Result<_, url::ParseError>>()?,
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
