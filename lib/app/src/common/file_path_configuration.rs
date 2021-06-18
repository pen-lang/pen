pub const OBJECT_DIRECTORY: &str = "objects";
pub const EXTERNAL_PACKAGES_DIRECTORY: &str = "packages";

pub const OBJECT_FILE_EXTENSION: &str = "bc";
pub const INTERFACE_FILE_EXTENSION: &str = "json";

pub struct FilePathConfiguration {
    pub package_configuration_filename: &'static str,
    pub output_directory_name: &'static str,
    pub source_file_extension: &'static str,
    pub main_file_basename: &'static str,
}
