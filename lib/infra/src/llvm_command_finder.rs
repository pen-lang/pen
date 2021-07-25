use crate::command_finder;
use std::{error::Error, path::PathBuf};

const MINIMUM_VERSION: usize = 11;
const MAXIMUM_VERSION: usize = 12;

pub fn find(command: &str) -> Result<PathBuf, Box<dyn Error>> {
    for version in (MINIMUM_VERSION..=MAXIMUM_VERSION).rev() {
        if let Ok(path) = command_finder::find(&format!("{}-{}", command, version)) {
            return Ok(path);
        }
    }

    command_finder::find(command)
}
