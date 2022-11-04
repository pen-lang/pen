use std::{env, ffi::OsString, path::Path, str};

fn main() {
    for (index, path) in env::var("PEN_OS_ARCHIVES").unwrap().split(":").enumerate() {
        let path = Path::new(path);

        if index == 0 {
            println!("cargo:rustc-link-lib=static={}", convert_path_to_flag(path));
        } else {
            println!("cargo:rustc-link-lib={}", convert_path_to_flag(path));
        }

        println!(
            "cargo:rustc-link-search={}",
            path.parent().unwrap().display()
        );
    }

    if env::var_os("CARGO_CFG_TARGET_OS") == Some(OsString::from("macos")) {
        println!("cargo:rustc-link-lib=framework=Foundation",);
        println!("cargo:rustc-link-lib=framework=Security",);
    }
}

fn convert_path_to_flag(path: &Path) -> &str {
    path.file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("lib")
}
