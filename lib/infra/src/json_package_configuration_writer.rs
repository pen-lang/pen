use super::json_package_configuration::JsonPackageConfiguration;
use std::{error::Error, sync::Arc};

pub struct JsonPackageConfigurationWriter {
    file_system: Arc<dyn app::infra::FileSystem>,
    build_configuration_filename: &'static str,
}

impl JsonPackageConfigurationWriter {
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

impl app::infra::PackageConfigurationWriter for JsonPackageConfigurationWriter {
    fn write(
        &self,
        package_configuration: &app::infra::PackageConfiguration,
        package_directory: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        self.file_system.write(
            &package_directory.join(&app::infra::FilePath::new(vec![
                self.build_configuration_filename,
            ])),
            &serde_json::to_vec_pretty::<JsonPackageConfiguration>(
                &package_configuration.clone().into(),
            )?,
        )?;

        Ok(())
    }
}
