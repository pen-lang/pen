use std::{
    collections::BTreeSet,
    env,
    error::Error,
    path::{self, Path},
};

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Box<dyn Error>> {
    let archive_files_string = env::var("PEN_ARCHIVE_FILES")?;
    let archive_files = archive_files_string.split(":").collect::<Vec<_>>();

    println!(
        "cargo:rustc-link-lib=static={}",
        get_library_name(&archive_files[0]),
    );

    for path in &archive_files[1..] {
        println!("cargo:rustc-link-lib={}", get_library_name(&path),);
    }

    for path in archive_files
        .iter()
        .map(|path| get_parent_directory(path).to_string())
        .collect::<BTreeSet<_>>()
    {
        println!("cargo:rustc-link-search={}", path);
    }

    Ok(())
}

fn get_library_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .strip_prefix("lib")
        .unwrap()
        .into()
}

fn get_parent_directory(path: &str) -> path::Display {
    Path::new(path).parent().unwrap().display()
}
