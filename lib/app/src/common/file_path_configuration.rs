#[derive(Clone, Debug)]
pub struct FilePathConfiguration {
    pub source_file_extension: &'static str,
    pub object_file_extension: &'static str,
    pub interface_file_extension: &'static str,
}
