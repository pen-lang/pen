use crate::{
    application_configuration::APPLICATION_CONFIGURATION,
    file_path_configuration::{
        DEFAULT_TARGET_DIRECTORY, FFI_PACKAGE_URL, OUTPUT_DIRECTORY, PRELUDE_PACKAGE_URL,
    },
    infrastructure, main_package_directory_finder,
};
use std::rc::Rc;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Rc::new(infra::FilePathConverter::new(&main_package_directory));
    let infrastructure =
        infrastructure::create(file_path_converter.clone(), &main_package_directory)?;
    let main_package_directory =
        file_path_converter.convert_to_file_path(&main_package_directory)?;
    let output_directory = main_package_directory.join(&app::infra::FilePath::new([
        OUTPUT_DIRECTORY,
        DEFAULT_TARGET_DIRECTORY,
    ]));

    app::package_initializer::initialize(
        &infrastructure,
        &main_package_directory,
        &output_directory,
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
        &url::Url::parse(FFI_PACKAGE_URL)?,
    )?;

    app::test_runner::run(
        &infrastructure,
        &main_package_directory,
        &output_directory,
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
        &url::Url::parse(FFI_PACKAGE_URL)?,
        &APPLICATION_CONFIGURATION,
    )?;

    Ok(())
}
