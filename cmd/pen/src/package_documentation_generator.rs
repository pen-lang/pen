use crate::{
    documentation_configuration::DOCUMENTATION_CONFIGURATION, infrastructure,
    main_package_directory_finder,
};
use app::package_documentation_generator::DocumentationPackage;
use std::{
    error::Error,
    io::{Write, stdout},
    rc::Rc,
};

pub fn generate(name: &str, url: &str, description: &str) -> Result<(), Box<dyn Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Rc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));

    stdout().write_all(
        app::package_documentation_generator::generate(
            &infrastructure::create(file_path_converter.clone(), &main_package_directory)?,
            &DocumentationPackage {
                name: name.into(),
                url: url.into(),
                description: description.into(),
            },
            &file_path_converter.convert_to_file_path(&main_package_directory)?,
            &DOCUMENTATION_CONFIGURATION,
        )?
        .as_bytes(),
    )?;

    Ok(())
}
