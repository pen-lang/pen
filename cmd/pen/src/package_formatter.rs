use crate::{infrastructure, main_package_directory_finder};
use std::{
    error::Error,
    io::{stdin, stdout, Read, Write},
    sync::Arc,
};

const STDIN_PATH: &str = "<stdin>";

pub fn format(stdin: bool) -> Result<(), Box<dyn Error>> {
    if stdin {
        return format_module();
    }

    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));
    let infrastructure =
        infrastructure::create(file_path_converter.clone(), &main_package_directory)?;

    app::package_formatter::format(
        &infrastructure,
        &file_path_converter.convert_to_file_path(&main_package_directory)?,
    )?;

    Ok(())
}

fn format_module() -> Result<(), Box<dyn Error>> {
    let mut source = String::new();

    stdin().read_to_string(&mut source)?;

    write!(
        stdout(),
        "{}",
        app::module_formatter::format(&source, STDIN_PATH)?
    )?;

    Ok(())
}
