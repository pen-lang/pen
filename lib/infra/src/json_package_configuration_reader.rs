use super::json_package_configuration::JsonPackageConfiguration;
use std::{convert::TryInto, error::Error, sync::Arc};

pub struct JsonPackageConfigurationReader {
    file_system: Arc<dyn app::infra::FileSystem>,
    build_configuration_filename: &'static str,
}

impl JsonPackageConfigurationReader {
    pub fn new(
        file_system: Arc<dyn app::infra::FileSystem>,
        build_configuration_filename: &'static str,
    ) -> Self {
        Self {
            file_system,
            build_configuration_filename,
        }
    }
}

impl app::infra::PackageConfigurationReader for JsonPackageConfigurationReader {
    fn read(
        &self,
        package_directory: &app::infra::FilePath,
    ) -> Result<app::infra::PackageConfiguration, Box<dyn Error>> {
        Ok(
            serde_json::from_str::<JsonPackageConfiguration>(&self.file_system.read_to_string(
                &package_directory.join(&app::infra::FilePath::new(vec![
                    self.build_configuration_filename,
                ])),
            )?)?
            .try_into()?,
        )
    }
}
