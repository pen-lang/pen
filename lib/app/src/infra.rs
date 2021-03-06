mod build_script_compiler;
mod build_script_dependency_compiler;
mod build_script_runner;
mod command_runner;
mod external_package_initializer;
mod file_path;
mod file_path_configuration;
mod file_path_displayer;
mod file_system;
mod infrastructure;
mod main_module_target;
mod module_target;
mod module_target_source;
mod package_configuration_reader;
mod package_configuration_writer;
mod test_linker;
mod test_module_target;

pub use build_script_compiler::*;
pub use build_script_dependency_compiler::*;
pub use build_script_runner::*;
pub use command_runner::*;
pub use external_package_initializer::*;
pub use file_path::*;
pub use file_path_configuration::*;
pub use file_path_displayer::*;
pub use file_system::*;
pub use infrastructure::*;
pub use main_module_target::*;
pub use module_target::*;
pub use module_target_source::*;
pub use package_configuration_reader::*;
pub use package_configuration_writer::*;
pub use test_linker::*;
pub use test_module_target::*;

pub const EXTERNAL_PACKAGE_DIRECTORY: &str = "packages";
pub const OBJECT_DIRECTORY: &str = "objects";
pub const ARCHIVE_DIRECTORY: &str = "archives";
pub const BUILD_SCRIPT_DIRECTORY: &str = "scripts";
pub const TEST_DIRECTORY: &str = "test";
