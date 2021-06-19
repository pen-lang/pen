use crate::common::FilePathConfiguration;
use crate::infra::{ModuleBuilder, FileSystem};
use std::sync::Arc;

pub struct BuildInfrastructure {
    pub module_builder: Arc<dyn ModuleBuilder>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
