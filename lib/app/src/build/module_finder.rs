use super::build_infrastructure::BuildInfrastructure;
use crate::infra::FilePath;

pub fn find_modules(
    infrastructure: &BuildInfrastructure,
    directory_path: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn std::error::Error>> {
    let mut source_file_paths = vec![];

    for path in infrastructure.file_system.read_directory(directory_path)? {
        if path
            .relative_to(directory_path)
            .components()
            .next()
            .unwrap()
            .starts_with('.')
        {
        } else if infrastructure.file_system.is_directory(&path) {
            source_file_paths.extend(find_modules(infrastructure, &path)?);
        } else if path.has_extension(infrastructure.file_path_configuration.source_file_extension) {
            source_file_paths.push(path);
        }
    }

    Ok(source_file_paths)
}
