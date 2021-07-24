use crate::{command_runner, llvm_command_finder, InfrastructureError};
use std::{error::Error, process::Command};

pub fn find() -> Result<String, Box<dyn Error>> {
    Ok(regex::Regex::new(r".*Default target: (.*)\n.*")?
        .captures(&command_runner::run(
            Command::new(&llvm_command_finder::find("llc")?).arg("--version"),
        )?)
        .and_then(|captures| captures.get(1))
        .ok_or(InfrastructureError::DefaultTargetDetection)?
        .as_str()
        .into())
}
