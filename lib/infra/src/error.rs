use std::{error::Error, fmt::Display, io, path::PathBuf};

#[derive(Debug)]
pub enum InfrastructureError {
    CommandExit { status_code: Option<i32> },
    CommandNotFound(String),
    CreateDirectory { path: PathBuf, source: io::Error },
    EnvironmentVariableNotFound(String),
    LinkScriptNotFound,
    PackageUrlSchemeNotSupported(url::Url),
    ReadDirectory { path: PathBuf, source: io::Error },
    ReadFile { path: PathBuf, source: io::Error },
    MultipleFfiBuildScripts(PathBuf),
    MultipleLinkScripts(Vec<PathBuf>),
    WriteFile { path: PathBuf, source: io::Error },
}

impl Error for InfrastructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandExit { status_code: _ } => None,
            Self::CommandNotFound(_) => None,
            Self::CreateDirectory { path: _, source } => Some(source),
            Self::EnvironmentVariableNotFound(_) => None,
            Self::LinkScriptNotFound => None,
            Self::PackageUrlSchemeNotSupported(_) => None,
            Self::ReadDirectory { path: _, source } => Some(source),
            Self::ReadFile { path: _, source } => Some(source),
            Self::MultipleFfiBuildScripts(_) => None,
            Self::MultipleLinkScripts(_) => None,
            Self::WriteFile { path: _, source } => Some(source),
        }
    }
}

impl Display for InfrastructureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::CommandExit { status_code } => match status_code {
                Some(status_code) => {
                    write!(formatter, "command exited with status code {status_code}")
                }
                None => write!(formatter, "command exited without status code"),
            },
            Self::CommandNotFound(command) => {
                write!(formatter, "command \"{command}\" not found")
            }
            Self::CreateDirectory { path, source: _ } => write!(
                formatter,
                "failed to create directory {}",
                path.to_string_lossy()
            ),
            Self::EnvironmentVariableNotFound(name) => {
                write!(formatter, "environment variable \"{name}\" not found")
            }
            Self::LinkScriptNotFound => {
                write!(formatter, "link script not found in any system packages")
            }
            Self::PackageUrlSchemeNotSupported(url) => {
                write!(formatter, "package URL scheme not supported {url}")
            }
            Self::ReadDirectory { path, source: _ } => write!(
                formatter,
                "failed to read directory {}",
                path.to_string_lossy()
            ),
            Self::ReadFile { path, source: _ } => {
                write!(formatter, "failed to read file {}", path.to_string_lossy())
            }
            Self::MultipleFfiBuildScripts(path) => {
                write!(
                    formatter,
                    "multiple FFI build scripts found in package directory \"{}\"",
                    path.display()
                )
            }
            Self::MultipleLinkScripts(paths) => {
                write!(
                    formatter,
                    "multiple link scripts found in system packages {}",
                    paths
                        .iter()
                        .map(|path| format!("\"{}\"", path.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::WriteFile { path, source: _ } => {
                write!(formatter, "failed to write file {}", path.to_string_lossy())
            }
        }
    }
}
