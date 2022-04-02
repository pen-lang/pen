use crate::{
    error::ApplicationError,
    infra::{FilePath, Infrastructure},
    module_finder, module_formatter,
};
use std::error::Error;

pub fn check(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let mut paths = vec![];

    for path in module_finder::find(infrastructure, package_directory)? {
        let source = infrastructure.file_system.read_to_string(&path)?;
        let path = infrastructure.file_path_displayer.display(&path);

        if source != module_formatter::format(&source, &path)? {
            paths.push(path);
        }
    }

    if paths.is_empty() {
        Ok(())
    } else {
        Err(ApplicationError::ModuleFilesNotFormatted(paths).into())
    }
}
