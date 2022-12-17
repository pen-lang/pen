use super::json_package_configuration::JsonPackageConfiguration;
use crate::FilePathConverter;
use std::{error::Error, sync::Arc};

pub struct JsonPackageConfigurationReader {
    file_system: Arc<dyn app::infra::FileSystem>,
    file_path_converter: Arc<FilePathConverter>,
    build_configuration_filename: &'static str,
}

impl JsonPackageConfigurationReader {
    pub fn new(
        file_system: Arc<dyn app::infra::FileSystem>,
        file_path_converter: Arc<FilePathConverter>,
        build_configuration_filename: &'static str,
    ) -> Self {
        Self {
            file_system,
            file_path_converter,
            build_configuration_filename,
        }
    }
}

impl app::infra::PackageConfigurationReader for JsonPackageConfigurationReader {
    fn read(
        &self,
        package_directory: &app::infra::FilePath,
    ) -> Result<app::PackageConfiguration, Box<dyn Error>> {
        let package_file_url = url::Url::from_directory_path(
            self.file_path_converter
                .convert_to_os_path(package_directory)
                .canonicalize()?,
        )
        .unwrap();

        Ok(
            serde_json::from_str::<JsonPackageConfiguration>(&self.file_system.read_to_string(
                &package_directory.join(&app::infra::FilePath::new(vec![
                    self.build_configuration_filename,
                ])),
            )?)?
            .into_configuration(&package_file_url)?,
        )
    }
}
