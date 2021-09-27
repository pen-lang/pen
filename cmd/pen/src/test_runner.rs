use crate::{
    file_path_configuration::{DEFAULT_TARGET_DIRECTORY, OUTPUT_DIRECTORY},
    infrastructure, main_package_directory_finder,
};
use std::sync::Arc;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));
    let main_package_directory =
        file_path_converter.convert_to_file_path(&main_package_directory)?;

    app::test_runner::run(
        &infrastructure::create(file_path_converter)?,
        &main_package_directory,
        &main_package_directory.join(&app::infra::FilePath::new([
            OUTPUT_DIRECTORY,
            DEFAULT_TARGET_DIRECTORY,
        ])),
    )?;

    Ok(())
}
