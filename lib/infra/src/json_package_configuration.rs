use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO Remove the allow when clippy::use_self's bug is fixed.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::use_self)]
#[serde(rename_all = "camelCase")]
pub enum JsonPackageType {
    Application,
    Library,
    System,
}

impl From<app::PackageType> for JsonPackageType {
    fn from(type_: app::PackageType) -> Self {
        match type_ {
            app::PackageType::Application => Self::Application,
            app::PackageType::Library => Self::Library,
            app::PackageType::System => Self::System,
        }
    }
}

impl From<JsonPackageType> for app::PackageType {
    fn from(type_: JsonPackageType) -> Self {
        match type_ {
            JsonPackageType::Application => Self::Application,
            JsonPackageType::Library => Self::Library,
            JsonPackageType::System => Self::System,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    #[serde(rename = "type")]
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
