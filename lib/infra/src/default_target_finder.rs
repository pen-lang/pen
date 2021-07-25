use crate::{command_runner, llvm_command_finder};
use std::{error::Error, process::Command};

pub fn find() -> Result<String, Box<dyn Error>> {
    let target = command_runner::run(
        Command::new(&llvm_command_finder::find("llvm-config")?).arg("--host-target"),
    )?;

    // HACK Map a given target to a known Rust target in the best effort way.
    Ok(target.replace("-pc-linux-gnu", "-unknown-linux-gnu"))
}
