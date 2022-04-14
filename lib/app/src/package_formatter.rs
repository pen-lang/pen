use crate::{
    infra::{FilePath, Infrastructure},
    module_finder, module_formatter, test_module_finder,
};
use std::error::Error;

pub fn format(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    for paths in module_finder::find(infrastructure, package_directory)?
        .into_iter()
        .chain(test_module_finder::find(infrastructure, package_directory)?)
    {
        infrastructure.file_system.write(
            &paths,
            module_formatter::format(
                &infrastructure.file_system.read_to_string(&paths)?,
                &infrastructure.file_path_displayer.display(&paths),
            )?
            .as_bytes(),
        )?;
    }

    Ok(())
}
