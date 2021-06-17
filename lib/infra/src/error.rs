use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum InfrastructureError {
    CreateDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    ReadDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    ReadFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    WriteFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

impl Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::CreateDirectory { path: _, source } => source,
            Self::ReadDirectory { path: _, source } => source,
            Self::ReadFile { path: _, source } => source,
            Self::WriteFile { path: _, source } => source,
        })
    }
}

impl Display for InfrastructureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::CreateDirectory { path, source: _ } => write!(
                formatter,
                "failed to create directory {}",
                path.to_string_lossy()
            ),
            Self::ReadDirectory { path, source: _ } => write!(
                formatter,
                "failed to read directory {}",
                path.to_string_lossy()
            ),
            Self::ReadFile { path, source: _ } => {
                write!(formatter, "failed to read file {}", path.to_string_lossy())
            }
            Self::WriteFile { path, source: _ } => {
                write!(formatter, "failed to write file {}", path.to_string_lossy())
            }
        }
    }
}
