use crate::{infrastructure, main_package_directory_finder};
use std::{
    error::Error,
    io::{stdout, Write},
    sync::Arc,
};

const LANGUAGE_TAG: &str = "pen";

pub fn generate(package_name: &str) -> Result<(), Box<dyn Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));

    stdout().write_all(
        app::package_documentation_generator::generate(
            &infrastructure::create(file_path_converter.clone(), &main_package_directory)?,
            package_name,
            &file_path_converter.convert_to_file_path(&main_package_directory)?,
            LANGUAGE_TAG,
        )?
        .as_bytes(),
    )?;

    Ok(())
}
