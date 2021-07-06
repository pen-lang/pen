mod application_configuration;
mod common;
pub mod infra;
pub mod module_compiler;
pub mod module_dependency_resolver;
mod module_finder;
mod package_build_script_compiler;
pub mod package_builder;
pub mod package_initializer;
mod prelude_interface_file_finder;

pub use application_configuration::{ApplicationConfiguration, MainModuleConfiguration};
