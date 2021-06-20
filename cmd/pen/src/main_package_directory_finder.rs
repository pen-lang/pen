use crate::file_path_configuration::BUILD_CONFIGURATION_FILENAME;

pub fn find() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mut directory: &std::path::Path = &std::env::current_dir()?;

    while !directory.join(BUILD_CONFIGURATION_FILENAME).exists() {
        directory = directory.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "file {} not found in any parent directory",
                    BUILD_CONFIGURATION_FILENAME,
                ),
            )
        })?
    }

    Ok(directory.into())
}
