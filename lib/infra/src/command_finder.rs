use crate::InfrastructureError;
use std::{error::Error, path::PathBuf};

pub fn find(command: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(which::which(command).map_err(|_| InfrastructureError::CommandNotFound(command.into()))?)
}
