use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageType {
    Application,
    Library,
    System,
}

impl Display for PackageType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::Application => "application",
                Self::Library => "library",
                Self::System => "system",
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageConfiguration {
    type_: PackageType,
    dependencies: BTreeMap<String, url::Url>,
}

impl PackageConfiguration {
    pub fn new(type_: PackageType, dependencies: BTreeMap<String, url::Url>) -> Self {
        Self {
            type_,
            dependencies,
        }
    }

    pub fn type_(&self) -> PackageType {
        self.type_
    }

    pub fn dependencies(&self) -> &BTreeMap<String, url::Url> {
        &self.dependencies
    }
}
