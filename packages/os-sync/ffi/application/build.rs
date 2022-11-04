use std::{env, path::Path, str};

fn main() {
    for (index, path) in env::var("PEN_OS_ARCHIVES")
        .iter()
        .flat_map(|paths| paths.split(':'))
        .enumerate()
    {
        let path = Path::new(path);
        let name = convert_path_to_name(path);

        if index == 0 {
            println!("cargo:rustc-link-lib=static={}", name);
        } else {
            println!("cargo:rustc-link-lib={}", name);
        }

        println!(
            "cargo:rustc-link-search={}",
            path.parent().unwrap().display()
        );
    }

    if cfg!(target_os = "macos") {
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
