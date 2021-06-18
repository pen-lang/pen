use crate::common::FilePathConfiguration;
use crate::infra::{Builder, FileSystem};
use std::sync::Arc;

pub struct BuildInfrastructure {
    pub builder: Arc<dyn Builder>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
