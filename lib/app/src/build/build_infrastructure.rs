use crate::common::FilePathConfiguration;
use crate::infra::FileSystem;
use std::sync::Arc;

pub struct BuildInfrastructure {
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
