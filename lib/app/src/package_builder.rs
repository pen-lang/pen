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
        application_configuration,
    )?;

    infrastructure.module_builder.build(&build_script_file)?;

    let files = infrastructure.file_system.read_directory(
        &file_path_resolver::resolve_object_directory(output_directory),
    )?;
    let (archive_files, files) = files.into_iter().partition::<Vec<_>, _>(|file| {
        file.has_extension(
            &infrastructure
                .file_path_configuration
                .archive_file_extension,
        )
    });
    let (object_files, _) = files.into_iter().partition::<Vec<_>, _>(|file| {
        file.has_extension(&infrastructure.file_path_configuration.object_file_extension)
    });

    if infrastructure
        .file_system
        .exists(&file_path_resolver::resolve_source_file(
            main_package_directory,
            &[application_configuration.main_module_basename.clone()],
            &infrastructure.file_path_configuration,
        ))
    {
        infrastructure.application_linker.link(
            &object_files,
            &archive_files,
            &main_package_directory.join(&FilePath::new(["app"])),
        )?;
    }

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
