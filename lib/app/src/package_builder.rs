use super::application_configuration::ApplicationConfiguration;
use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, EXTERNAL_PACKAGE_DIRECTORY},
    package_build_script_compiler,
};
use std::error::Error;

pub fn build(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    target_triple: Option<&str>,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let file_extension = infrastructure
        .file_path_configuration
        .build_script_file_extension;
    let rule_build_script_file =
        output_directory.join(&FilePath::new(vec!["rules"]).with_extension(file_extension));
    let package_build_script_file =
        output_directory.join(&FilePath::new(vec!["main"]).with_extension(file_extension));

    package_build_script_compiler::compile_rules(
        infrastructure,
        prelude_package_url,
        output_directory,
        target_triple,
        &rule_build_script_file,
    )?;

    package_build_script_compiler::compile_main(
        infrastructure,
        main_package_directory,
        output_directory,
        &rule_build_script_file,
        &find_external_package_build_scripts(infrastructure, output_directory)?
            .into_iter()
            .chain(
                if is_application_package(
                    infrastructure,
                    main_package_directory,
                    application_configuration,
                )? {
                    let application_build_script_file = output_directory
                        .join(&FilePath::new(vec!["application"]).with_extension(file_extension));

                    package_build_script_compiler::compile_application(
                        infrastructure,
                        main_package_directory,
                        output_directory,
                        prelude_package_url,
                        application_configuration,
                        &application_build_script_file,
                    )?;

                    Some(application_build_script_file)
                } else {
                    None
                },
            )
            .collect::<Vec<_>>(),
        &package_build_script_file,
        application_configuration,
    )?;

    infrastructure
        .package_builder
        .build(&package_build_script_file)?;

    Ok(())
}

fn is_application_package(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    application_configuration: &ApplicationConfiguration,
) -> Result<bool, Box<dyn Error>> {
    Ok(infrastructure
        .file_system
        .exists(&file_path_resolver::resolve_source_file(
            main_package_directory,
            &[application_configuration.main_module_basename.clone()],
            &infrastructure.file_path_configuration,
        )))
}

fn find_external_package_build_scripts(
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
