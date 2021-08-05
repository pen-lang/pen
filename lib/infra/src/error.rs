use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum InfrastructureError {
    CommandExit {
        status_code: Option<i32>,
    },
    CommandNotFound(String),
    CreateDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    EnvironmentVariableNotFound(String),
    PackageUrlSchemeNotSupported(url::Url),
    ReadDirectory {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    ReadFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    TooManyFfiBuildScripts(std::path::PathBuf),
    WriteFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

impl Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandExit { status_code: _ } => None,
            Self::CommandNotFound(_) => None,
            Self::CreateDirectory { path: _, source } => Some(source),
            Self::EnvironmentVariableNotFound(_) => None,
            Self::PackageUrlSchemeNotSupported(_) => None,
            Self::ReadDirectory { path: _, source } => Some(source),
            Self::ReadFile { path: _, source } => Some(source),
            Self::TooManyFfiBuildScripts(_) => None,
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
            Self::CommandNotFound(command) => {
                write!(formatter, "command \"{}\" not found", command)
            }
            Self::CreateDirectory { path, source: _ } => write!(
                formatter,
                "failed to create directory {}",
                path.to_string_lossy()
            ),
            Self::EnvironmentVariableNotFound(name) => {
                write!(formatter, "environment variable \"{}\" not found", name)
            }
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
            Self::TooManyFfiBuildScripts(path) => {
                write!(
                    formatter,
                    "too many FFI build scripts in directory \"{}\"",
                    path.display()
                )
            }
            Self::WriteFile { path, source: _ } => {
                write!(formatter, "failed to write file {}", path.to_string_lossy())
            }
        }
    }
}
