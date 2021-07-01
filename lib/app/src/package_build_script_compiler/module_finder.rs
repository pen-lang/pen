use crate::infra::{FilePath, Infrastructure};
use std::error::Error;

pub fn find(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
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
            source_files.extend(find(infrastructure, &path)?);
        } else if path.has_extension(infrastructure.file_path_configuration.source_file_extension) {
            source_files.push(path);
        }
    }

    Ok(source_files)
}
