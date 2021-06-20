use crate::{
    infra::FilePathConfiguration,
    infra::{FileSystem, ModuleBuilder},
};
use std::sync::Arc;

pub struct PackageBuilderInfrastructure {
    pub module_builder: Arc<dyn ModuleBuilder>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
