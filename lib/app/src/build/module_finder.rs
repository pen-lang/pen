use super::build_infrastructure::BuildInfrastructure;
use crate::infra::FilePath;

pub fn find_modules(
    infrastructure: &BuildInfrastructure,
    directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn std::error::Error>> {
    let mut source_files = vec![];

    for path in infrastructure.file_system.read_directory(directory)? {
        if path
            .relative_to(directory)
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
