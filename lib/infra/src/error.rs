use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum InfrastructureError {
    CommandExit {
        status_code: Option<i32>,
    },
    CreateDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    PackageUrlSchemeNotSupported(url::Url),
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
        match self {
            Self::CommandExit { status_code: _ } => None,
            Self::CreateDirectory { path: _, source } => Some(source),
            Self::PackageUrlSchemeNotSupported(_) => None,
            Self::ReadDirectory { path: _, source } => Some(source),
            Self::ReadFile { path: _, source } => Some(source),
            Self::WriteFile { path: _, source } => Some(source),
        }
    }
}

impl Display for InfrastructureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::CommandExit { status_code } => match status_code {
                Some(status_code) => {
                    write!(formatter, "command exited with status code {}", status_code)
                }
                None => write!(formatter, "command exited without status code"),
            },
            Self::CreateDirectory { path, source: _ } => write!(
                formatter,
                "failed to create directory {}",
                path.to_string_lossy()
            ),
            Self::PackageUrlSchemeNotSupported(url) => {
                write!(formatter, "package URL scheme not supported {}", url)
            }
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
