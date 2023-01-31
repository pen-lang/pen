use std::{env, path::Path, str};

const ARCHIVE_VARIABLE_NAME: &str = "PEN_TEST_ARCHIVES";

fn main() {
    println!("cargo:rerun-if-env-changed={ARCHIVE_VARIABLE_NAME}");

    for (index, path) in env::var(ARCHIVE_VARIABLE_NAME)
        .iter()
        .flat_map(|paths| paths.split(':'))
        .enumerate()
    {
        let path = Path::new(path);
        let name = convert_path_to_name(path);

        if index == 0 {
            println!("cargo:rustc-link-lib=static={name}");
        } else {
            println!("cargo:rustc-link-lib={name}");
        }

        println!("cargo:rerun-if-changed={name}");
        println!(
            "cargo:rustc-link-search={}",
            path.parent().unwrap().display()
        );
    }

    // https://github.com/rust-lang/cargo/issues/4932
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        println!("cargo:rustc-link-lib=framework=Foundation",);
        println!("cargo:rustc-link-lib=framework=Security",);
    }
}

fn convert_path_to_name(path: &Path) -> &str {
    path.file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("lib")
}
