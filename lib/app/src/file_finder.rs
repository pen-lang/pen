use crate::infra::{FilePath, Infrastructure};
use std::error::Error;

pub fn find(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    file_extension: &str,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
    let mut source_files = vec![];

    for path in infrastructure
        .file_system
        .read_directory(package_directory)?
    {
        if path
            .relative_to(package_directory)
            .unwrap()
            .components()
            .next()
            .unwrap()
            .starts_with('.')
        {
            continue;
        } else if infrastructure.file_system.is_directory(&path) {
            source_files.extend(find(infrastructure, &path, file_extension)?);
        } else if path.has_extension(file_extension) {
            source_files.push(path);
        }
    }

    Ok(source_files)
}
