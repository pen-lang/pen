use crate::{infrastructure, main_package_directory_finder};
use std::{error::Error, sync::Arc};

pub fn format(check: bool) -> Result<(), Box<dyn Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));
    let infrastructure =
        infrastructure::create(file_path_converter.clone(), &main_package_directory)?;

    if check {
        app::package_format_checker::check(
            &infrastructure,
            &file_path_converter.convert_to_file_path(&main_package_directory)?,
        )?;
    } else {
        app::package_formatter::format(
            &infrastructure,
            &file_path_converter.convert_to_file_path(&main_package_directory)?,
        )?;
    }

    Ok(())
}
