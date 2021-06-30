mod build_script_dependency_compiler;
mod external_package_initializer;
mod file_path;
mod file_path_configuration;
mod file_path_displayer;
mod file_system;
mod module_build_script_compiler;
mod module_builder;
mod module_target;
mod package_configuration;
mod package_configuration_reader;
mod prelude_package_configuration;

pub use build_script_dependency_compiler::*;
pub use external_package_initializer::*;
pub use file_path::*;
pub use file_path_configuration::*;
pub use file_path_displayer::*;
pub use file_system::*;
pub use module_build_script_compiler::*;
pub use module_builder::*;
pub use module_target::*;
pub use package_configuration::*;
pub use package_configuration_reader::*;
pub use prelude_package_configuration::*;

pub const EXTERNAL_PACKAGE_DIRECTORY: &str = "packages";
pub const OBJECT_DIRECTORY: &str = "objects";
