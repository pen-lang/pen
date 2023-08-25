use crate::{
    application_configuration::APPLICATION_CONFIGURATION,
    file_path_configuration::{DEFAULT_SYSTEM_PACKAGE_NAME, DEFAULT_SYSTEM_PACKAGE_URL},
    infrastructure,
};
use std::{path::PathBuf, rc::Rc};

pub fn create(package_directory: &str, library: bool) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Rc::new(infra::FilePathConverter::new(
        PathBuf::from(package_directory).parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "parent directory not found")
        })?,
    ));
    let infrastructure = infrastructure::create(file_path_converter.clone(), package_directory)?;
    let package_directory = file_path_converter.convert_to_file_path(package_directory)?;

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
                import Os'File

                main = \\(ctx context) none {
                  _ = File'Write(ctx.Os, File'StdOut(), \"Hello, world!\\n\")

                  none
                }
                "
            ),
            DEFAULT_SYSTEM_PACKAGE_NAME,
            &url::Url::parse(DEFAULT_SYSTEM_PACKAGE_URL)?,
            &APPLICATION_CONFIGURATION,
            &package_directory,
        )?;
    }

    Ok(())
}
