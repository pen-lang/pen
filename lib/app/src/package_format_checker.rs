use crate::{
    error::ApplicationError,
    infra::{FilePath, Infrastructure},
    module_finder, module_formatter,
};
use std::error::Error;

pub fn format(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    for paths in module_finder::find(infrastructure, package_directory)? {
        let path = infrastructure.file_path_displayer.display(&paths);
        let source = infrastructure.file_system.read_to_string(&paths)?;
        let formatted_source = module_formatter::format(&source, &path)?;

        if source != formatted_source {
            return Err(ApplicationError::Format { path, difference });
        }
    }

    Ok(())
}
