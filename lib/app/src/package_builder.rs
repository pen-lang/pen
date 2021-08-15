use super::application_configuration::ApplicationConfiguration;
use crate::{
    common::file_path_resolver,
    error::ApplicationError,
    external_package_archive_sorter,
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
        &find_external_package_build_scripts(infrastructure, output_directory)?,
        &build_script_file,
        target_triple,
        prelude_package_url,
        application_configuration,
    )?;

    infrastructure.module_builder.build(&build_script_file)?;

    if is_main_package(
        infrastructure,
        main_package_directory,
        application_configuration,
    )? {
        infrastructure.application_linker.link(
            &file_path_resolver::resolve_package_directory(
                output_directory,
                infrastructure
                    .package_configuration_reader
                    .read(main_package_directory)?
                    .dependencies
                    .get(&application_configuration.system_package_name)
                    .ok_or(ApplicationError::SystemPackageNotFound)?,
            ),
            &vec![file_path_resolver::resolve_main_package_archive_file(
                output_directory,
                &infrastructure.file_path_configuration,
            )]
            .into_iter()
            .chain(ensure_file(
                infrastructure,
                &file_path_resolver::resolve_main_package_ffi_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
            ))
            .chain(
                external_package_archive_sorter::sort(
                    infrastructure,
                    main_package_directory,
                    output_directory,
                )?
                .iter()
                .flat_map(|url| {
                    resolve_external_package_archive_files(infrastructure, url, output_directory)
                }),
            )
            .chain(resolve_external_package_archive_files(
                infrastructure,
                prelude_package_url,
                output_directory,
            ))
            .collect::<Vec<_>>(),
            &main_package_directory.join(&FilePath::new([
                &application_configuration.application_filename
            ])),
            target_triple,
        )?;
    }

    Ok(())
}

fn is_main_package(
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

fn resolve_external_package_archive_files(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
) -> Vec<FilePath> {
    vec![file_path_resolver::resolve_external_package_archive_file(
        output_directory,
        package_url,
        &infrastructure.file_path_configuration,
    )]
    .into_iter()
    .chain(ensure_file(
        infrastructure,
        &file_path_resolver::resolve_external_package_ffi_archive_file(
            output_directory,
            package_url,
            &infrastructure.file_path_configuration,
        ),
    ))
    .collect()
}

fn ensure_file(infrastructure: &Infrastructure, ffi_archive_file: &FilePath) -> Option<FilePath> {
    if infrastructure.file_system.exists(ffi_archive_file) {
        Some(ffi_archive_file.clone())
    } else {
        None
    }
}
