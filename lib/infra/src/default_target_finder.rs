use crate::{command_runner, llvm_command_finder};
use std::{error::Error, process::Command};

pub fn find() -> Result<String, Box<dyn Error>> {
    Ok(replace_target(
        command_runner::run_command(
            Command::new(llvm_command_finder::find("llvm-config")?).arg("--host-target"),
        )?
        .trim(),
    ))
}

// HACK Map a given target to a known Rust target in the best effort way.
fn replace_target(target: &str) -> String {
    regex::Regex::new("-darwin.*")
        .unwrap()
        .replace(
            &target
                .replace("-pc-linux-gnu", "-unknown-linux-gnu")
                .replace("arm64-", "aarch64-"),
            "-darwin",
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_pc_for_linux_target() {
        assert_eq!(
            &replace_target("x86_64-pc-linux-gnu"),
            "x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn remove_darwin_version_for_apple_target() {
        assert_eq!(
            &replace_target("x86_64-apple-darwin19"),
            "x86_64-apple-darwin"
        );
    }

    #[test]
    fn replace_64_bit_arm_isa() {
        assert_eq!(
            &replace_target("arm64-apple-darwin"),
            "aarch64-apple-darwin"
        );
    }
}
