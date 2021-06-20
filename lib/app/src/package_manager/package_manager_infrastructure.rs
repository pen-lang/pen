use crate::infra::{ExternalPackageInitializer, FileSystem, PackageConfigurationReader};
use std::sync::Arc;

pub struct PackageManagerInfrastructure {
    pub external_package_initializer: Arc<dyn ExternalPackageInitializer>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
    pub file_system: Arc<dyn FileSystem>,
}
