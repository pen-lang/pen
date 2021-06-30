use super::package_build_script_compiler_infrastructure::PackageBuildScriptCompilerInfrastructure;
use crate::infra::FilePath;

pub fn find_modules(
    infrastructure: &PackageBuildScriptCompilerInfrastructure,
    package_directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn std::error::Error>> {
    let mut source_files = vec![];

    for path in infrastructure
        .file_system
        .read_directory(package_directory)?
    {
        if path
            .relative_to(package_directory)
            .components()
            .next()
            .unwrap()
            .starts_with('.')
        {
        } else if infrastructure.file_system.is_directory(&path) {
            source_files.extend(find_modules(infrastructure, &path)?);
        } else if path.has_extension(infrastructure.file_path_configuration.source_file_extension) {
            source_files.push(path);
        }
    }

    Ok(source_files)
}
