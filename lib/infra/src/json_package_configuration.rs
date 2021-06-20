use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonPackageConfiguration {
    pub dependencies: HashMap<String, String>,
}

impl TryFrom<JsonPackageConfiguration> for app::infra::PackageConfiguration {
    type Error = url::ParseError;

    fn try_from(configuration: JsonPackageConfiguration) -> Result<Self, url::ParseError> {
        Ok(app::infra::PackageConfiguration {
            dependencies: configuration
                .dependencies
                .iter()
                .map(|(name, url_string)| Ok((name.clone(), url::Url::parse(url_string)?)))
                .collect::<Result<HashMap<_, _>, url::ParseError>>()?,
        })
    }
}
