use super::application_configuration::ApplicationConfiguration;
use crate::{
    infra::{FilePath, Infrastructure, EXTERNAL_PACKAGE_DIRECTORY},
    package_build_script_compiler,
};
use std::error::Error;

pub fn build_main_package(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let build_script_file = output_directory.join(
        &FilePath::new(vec!["main"]).with_extension(
            infrastructure
                .file_path_configuration
                .build_script_file_extension,
        ),
    );

    package_build_script_compiler::compile_main(
        infrastructure,
        main_package_directory,
        output_directory,
        &find_external_package_build_script(infrastructure, output_directory)?,
        &build_script_file,
        prelude_package_url,
        is_package_application(
            infrastructure,
            main_package_directory,
            application_configuration,
        ),
    )?;

    infrastructure.module_builder.build(&build_script_file)?;

    Ok(())
}

fn find_external_package_build_script(
    infrastructure: &Infrastructure,
    output_directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
    let external_package_directory =
        output_directory.join(&FilePath::new(vec![EXTERNAL_PACKAGE_DIRECTORY]));

    Ok(
        if infrastructure
            .file_system
            .exists(&external_package_directory)
        {
            infrastructure
                .file_system
                .read_directory(&external_package_directory)?
                .into_iter()
                .filter(|path| {
                    path.has_extension(
                        infrastructure
                            .file_path_configuration
                            .build_script_file_extension,
                    )
                })
                .collect()
        } else {
            vec![]
        },
    )
}

fn is_package_application(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    application_configuration: &ApplicationConfiguration,
) -> bool {
    infrastructure.file_system.exists(
        &package_directory
            .join(&FilePath::new(vec![
                &application_configuration.main_module_basename,
            ]))
            .with_extension(infrastructure.file_path_configuration.source_file_extension),
    )
}
