mod module_target_collector;
mod test_module_target_collector;

use crate::{
    common::file_path_resolver,
    external_package_configuration_reader, external_package_topological_sorter,
    infra::{FilePath, Infrastructure, MainModuleTarget},
    prelude_interface_file_finder, system_package_finder, ApplicationConfiguration,
};
use std::error::Error;

// Compile a "main" build script that triggers build of a main package.
pub fn compile_main(
    infrastructure: &Infrastructure,
    prelude_package_url: &url::Url,
    output_directory: &FilePath,
    target_triple: Option<&str>,
    child_build_script_files: &[FilePath],
) -> Result<FilePath, Box<dyn Error>> {
    let build_script_file = file_path_resolver::resolve_special_build_script_file(
        output_directory,
        "main",
        &infrastructure.file_path_configuration,
    );

    infrastructure.file_system.write(
        &build_script_file,
        infrastructure
            .build_script_compiler
            .compile_main(
                &prelude_interface_file_finder::find(
                    infrastructure,
                    output_directory,
                    prelude_package_url,
                )?,
                output_directory,
                target_triple,
                child_build_script_files,
            )?
            .as_bytes(),
    )?;

    Ok(build_script_file)
}

pub fn compile_modules(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    application_configuration: &ApplicationConfiguration,
) -> Result<FilePath, Box<dyn Error>> {
    let build_script_file = file_path_resolver::resolve_special_build_script_file(
        output_directory,
        "modules",
        &infrastructure.file_path_configuration,
    );

    let (main_module_targets, module_targets) = module_target_collector::collect_module_targets(
        infrastructure,
        package_directory,
        output_directory,
    )?
    .into_iter()
    .partition::<Vec<_>, _>(|target| {
        target.source_file()
            == &file_path_resolver::resolve_source_file(
                package_directory,
                &[application_configuration.main_module_basename.clone()],
                &infrastructure.file_path_configuration,
            )
    });

    infrastructure.file_system.write(
        &build_script_file,
        infrastructure
            .build_script_compiler
            .compile_modules(
                &module_targets,
                main_module_targets
                    .get(0)
                    .map(|target| -> Result<_, Box<dyn Error>> {
                        Ok(MainModuleTarget::new(
                            target.source_file().clone(),
                            target.object_file().clone(),
                            system_package_finder::find(
                                infrastructure,
                                package_directory,
                                output_directory,
                            )?
                            .into_iter()
                            .map(|(key, url)| {
                                (
                                    key,
                                    file_path_resolver::resolve_interface_file(
                                        output_directory,
                                        &file_path_resolver::resolve_source_file(
                                            &file_path_resolver::resolve_package_directory(
                                                output_directory,
                                                &url,
                                            ),
                                            &[application_configuration
                                                .context_module_basename
                                                .clone()],
                                            &infrastructure.file_path_configuration,
                                        ),
                                        &infrastructure.file_path_configuration,
                                    ),
                                )
                            })
                            .collect(),
                        ))
                    })
                    .transpose()?
                    .as_ref(),
                &file_path_resolver::resolve_main_package_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
                package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(build_script_file)
}

pub fn compile_test_modules(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<FilePath, Box<dyn Error>> {
    let build_script_file = file_path_resolver::resolve_special_build_script_file(
        output_directory,
        "test_modules",
        &infrastructure.file_path_configuration,
    );

    infrastructure.file_system.write(
        &build_script_file,
        infrastructure
            .build_script_compiler
            .compile_test_modules(
                &test_module_target_collector::collect(
                    infrastructure,
                    package_directory,
                    output_directory,
                )?,
                &file_path_resolver::resolve_main_package_test_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
                &file_path_resolver::resolve_package_test_information_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
                package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(build_script_file)
}

pub fn compile_application(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<FilePath, Box<dyn Error>> {
    let external_package_configurations = external_package_configuration_reader::read_all(
        infrastructure,
        main_package_directory,
        output_directory,
    )?;
    let build_script_file = file_path_resolver::resolve_special_build_script_file(
        output_directory,
        "application",
        &infrastructure.file_path_configuration,
    );

    infrastructure.file_system.write(
        &build_script_file,
        infrastructure
            .build_script_compiler
            .compile_application(
                &system_package_finder::find(
                    infrastructure,
                    main_package_directory,
                    output_directory,
                )?
                .values()
                .map(|url| file_path_resolver::resolve_package_directory(output_directory, url))
                .collect::<Vec<_>>(),
                &[file_path_resolver::resolve_main_package_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                )]
                .into_iter()
                .chain(
                    external_package_topological_sorter::sort(&external_package_configurations)?
                        .iter()
                        .map(|url| {
                            file_path_resolver::resolve_external_package_archive_file(
                                output_directory,
                                url,
                                &infrastructure.file_path_configuration,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .chain(resolve_default_package_archive_files(
                    infrastructure,
                    output_directory,
                    prelude_package_url,
                    ffi_package_url,
                ))
                .collect::<Vec<_>>(),
                &main_package_directory.join(&FilePath::new([
                    &application_configuration.application_filename
                ])),
            )?
            .as_bytes(),
    )?;

    Ok(build_script_file)
}

pub fn compile_test(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
) -> Result<FilePath, Box<dyn Error>> {
    let build_script_file = file_path_resolver::resolve_special_build_script_file(
        output_directory,
        "test",
        &infrastructure.file_path_configuration,
    );

    infrastructure.file_system.write(
        &build_script_file,
        infrastructure
            .build_script_compiler
            .compile_test(
                &[
                    file_path_resolver::resolve_main_package_test_archive_file(
                        output_directory,
                        &infrastructure.file_path_configuration,
                    ),
                    file_path_resolver::resolve_main_package_archive_file(
                        output_directory,
                        &infrastructure.file_path_configuration,
                    ),
                ]
                .into_iter()
                .chain(
                    external_package_topological_sorter::sort(
                        &external_package_configuration_reader::read_all(
                            infrastructure,
                            main_package_directory,
                            output_directory,
                        )?,
                    )?
                    .iter()
                    .map(|url| {
                        file_path_resolver::resolve_external_package_archive_file(
                            output_directory,
                            url,
                            &infrastructure.file_path_configuration,
                        )
                    })
                    .collect::<Vec<_>>(),
                )
                .chain(resolve_default_package_archive_files(
                    infrastructure,
                    output_directory,
                    prelude_package_url,
                    ffi_package_url,
                ))
                .collect::<Vec<_>>(),
                &file_path_resolver::resolve_package_test_information_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
                &file_path_resolver::resolve_test_executable_file(output_directory),
            )?
            .as_bytes(),
    )?;

    Ok(build_script_file)
}

pub fn compile_external(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_external_package(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    &package_directory,
                    output_directory,
                )?,
                &file_path_resolver::resolve_external_package_archive_file(
                    output_directory,
                    package_url,
                    &infrastructure.file_path_configuration,
                ),
                &package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_prelude_package(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    &package_directory,
                    output_directory,
                )?,
                &file_path_resolver::resolve_external_package_archive_file(
                    output_directory,
                    package_url,
                    &infrastructure.file_path_configuration,
                ),
                &package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}

fn resolve_default_package_archive_files(
    infrastructure: &Infrastructure,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
) -> Vec<FilePath> {
    [prelude_package_url, ffi_package_url]
        .into_iter()
        .map(|url| {
            file_path_resolver::resolve_external_package_archive_file(
                output_directory,
                url,
                &infrastructure.file_path_configuration,
            )
        })
        .collect()
}
