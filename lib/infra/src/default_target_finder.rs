use crate::{command_runner, llvm_command_finder, InfrastructureError};
use std::{error::Error, process::Command};

pub fn find() -> Result<String, Box<dyn Error>> {
    let output =
        command_runner::run(Command::new(&llvm_command_finder::find("llc")?).arg("--version"))?;
    let target = regex::Regex::new(r".*Default target: (.*)\n.*")?
        .captures(&output)
        .and_then(|captures| captures.get(1))
        .ok_or(InfrastructureError::DefaultTargetDetection)?
        .as_str();

    // HACK Map a given target to a known Rust target in the best effort way.
    Ok(target.replace("-pc-linux-gnu", "-unknown-linux-gnu"))
}
