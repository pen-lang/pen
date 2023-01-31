use crate::command_finder;
use std::{error::Error, path::PathBuf};

const LLVM_VERSION: usize = 14;

pub fn find(command: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let Ok(path) = command_finder::find(&format!("{command}-{LLVM_VERSION}")) {
        return Ok(path);
    }

    command_finder::find(command)
}
