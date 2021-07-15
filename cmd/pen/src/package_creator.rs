use crate::{
    application_configuration::APPLICATION_CONFIGURATION,
    file_path_configuration::{DEFAULT_SYSTEM_PACKAGE_URL, PRELUDE_PACKAGE_URL},
    infrastructure,
};
use std::{path::PathBuf, sync::Arc};

pub fn create(package_directory: &str, library: bool) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        PathBuf::from(package_directory).parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "parent directory not found")
        })?,
    ));
    let infrastructure = infrastructure::create(file_path_converter.clone())?;
    let package_directory = file_path_converter.convert_to_file_path(&package_directory)?;

    if library {
        app::package_creator::create_library(
            &infrastructure,
            "Foo",
            indoc::indoc!(
                "
                Add = \\(x number, y number) number {
                  x + y
                }
                "
            ),
            &package_directory,
        )?;
    } else {
        app::package_creator::create_application(
            &infrastructure,
            indoc::indoc!(
                "
                import System'Os

                main = \\(os Os'Os) number {
                  Os'FdWrite(os, Os'StdOut(), \"Hello, world!\")

                  0
                }
                "
            ),
            &url::Url::parse(DEFAULT_SYSTEM_PACKAGE_URL)?,
            &APPLICATION_CONFIGURATION,
            &package_directory,
        )?;
    }

    Ok(())
}
