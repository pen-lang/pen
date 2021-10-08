use super::json_package_configuration::JsonPackageConfiguration;
use std::{collections::HashMap, error::Error, sync::Arc};

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
        dependencies: &HashMap<String, url::Url>,
        package_directory: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        self.file_system.write(
            &package_directory.join(&app::infra::FilePath::new(vec![
                self.build_configuration_filename,
            ])),
            (serde_json::to_string_pretty(&JsonPackageConfiguration::new(dependencies.clone()))?
                + "\n")
                .as_bytes(),
        )?;

        Ok(())
    }
}
