use crate::InfrastructureError;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub fn find(
    package_directory: &Path,
    script_basename: &str,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let ffi_build_scripts =
        glob::glob(&(package_directory.join(script_basename).to_string_lossy() + ".*"))?
            .collect::<Result<Vec<_>, _>>()?;

    Ok(match ffi_build_scripts.as_slice() {
        [] => None,
        [script] => Some(script.into()),
        _ => {
            return Err(
                InfrastructureError::MultipleFfiBuildScripts(package_directory.into()).into(),
            );
        }
    })
}
