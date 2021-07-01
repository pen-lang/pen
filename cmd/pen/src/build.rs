use super::main_package_directory_finder;
use crate::{
    file_path_configuration::{OUTPUT_DIRECTORY, PRELUDE_PACKAGE_URL},
    infrastructure,
};
use std::sync::Arc;

pub fn build(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(main_package_directory));
    let infrastructure =
        infrastructure::create(file_path_converter.clone(), &main_package_directory)?;
    let main_package_directory =
        file_path_converter.convert_to_file_path(&main_package_directory)?;
    let output_directory =
        main_package_directory.join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY]));

    if verbose {
        infra::log_info("initializing external packages")?;
    }

    app::package_initializer::initialize(
        &infrastructure,
        &main_package_directory,
        &output_directory,
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
    )?;

    if verbose {
        infra::log_info("building modules")?;
    }

    app::package_builder::build_main_package(
        &infrastructure,
        &main_package_directory,
        &output_directory,
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
    )?;

    Ok(())
}
