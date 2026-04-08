use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub struct FfiCrateInfo {
    directory: PathBuf,
    library_name: String,
}

impl FfiCrateInfo {
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn library_name(&self) -> &str {
        &self.library_name
    }
}

pub fn find(package_directory: &Path) -> Result<Option<FfiCrateInfo>, Box<dyn Error>> {
    let ffi_directory = package_directory.join("ffi");

    if ffi_directory.join("Cargo.toml").is_file() && is_package(&ffi_directory)? {
        return parse_crate_info(&ffi_directory);
    }

    let library_directory = ffi_directory.join("library");

    if library_directory.join("Cargo.toml").is_file() {
        return parse_crate_info(&library_directory);
    }

    Ok(None)
}

fn is_package(directory: &Path) -> Result<bool, Box<dyn Error>> {
    Ok(fs::read_to_string(directory.join("Cargo.toml"))?.contains("[package]"))
}

fn parse_crate_info(directory: &Path) -> Result<Option<FfiCrateInfo>, Box<dyn Error>> {
    let content = fs::read_to_string(directory.join("Cargo.toml"))?;

    let Some(name) = parse_package_name(&content) else {
        return Ok(None);
    };

    Ok(Some(FfiCrateInfo {
        directory: directory.into(),
        library_name: format!("lib{}.a", name.replace('-', "_")),
    }))
}

fn parse_package_name(content: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.trim().starts_with("name"))
        .and_then(|line| {
            line.split('=')
                .nth(1)
                .map(|value| value.trim().trim_matches('"').to_owned())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name() {
        assert_eq!(
            parse_package_name("[package]\nname = \"pen-html-ffi\"").as_deref(),
            Some("pen-html-ffi")
        );
    }

    #[test]
    fn convert_hyphens_to_underscores_in_library_name() {
        let info = parse_crate_info(Path::new("../../packages/html/ffi"))
            .unwrap()
            .unwrap();

        assert_eq!(info.library_name(), "libpen_html_ffi.a");
    }
}
