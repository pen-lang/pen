use crate::command_runner;
use std::{error::Error, path::PathBuf, process::Command};

pub fn find() -> Result<String, Box<dyn Error>> {
    let target = command_runner::run(Command::new(&find_llvm_config()?).arg("--host-target"))?;

    // HACK Map a given target to a known Rust target in the best effort way.
    Ok(target.replace("-pc-linux-gnu", "-unknown-linux-gnu"))
}

fn find_llvm_config() -> Result<PathBuf, Box<dyn Error>> {
    Ok(which::which("llvm-config")?)
}
