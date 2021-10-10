use std::fs::{self, OpenOptions};
use std::io::Write;
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

    write_main_rs()?;

    Ok(())
}

fn write_main_rs() -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("src/main.rs")?;

    write!(
        file,
        r#"
        mod heap;
        mod test_result;
        mod unreachable;
        use test_result::TestResult;
        extern "C" {{
            fn _pen_test_convert_result(result: ffi::Any) -> ffi::Arc<TestResult>;
        }}

        fn main() {{
            #[allow(unused_mut)]
            let mut success: usize = 0;
            #[allow(unused_mut)]
            let mut error: usize = 0;

            {}

            println!("test summary");
            println!(
                "\t{{}}\t{{}} passed, {{}} failed",
                if error == 0 {{ "OK" }} else {{ "FAIL" }},
                success, error
            );

            if error > 0 {{
                std::process::exit(1);
            }}
        }}
        "#,
        format_tests()?,
    )?;

    Ok(())
}

fn format_tests() -> Result<String, Box<dyn Error>> {
    let test_information = json::parse(&fs::read_to_string(&env::var(
        "PEN_TEST_INFORMATION_FILE",
    )?)?)?;

    Ok(test_information["modules"]
        .entries()
        .map(|(name, module)| {
            format!(r#"println!("{}");"#, name)
                + &module["functions"]
                    .members()
                    .map(|function| {
                        format_test_function(
                            function["name"].as_str().unwrap(),
                            function["foreign_name"].as_str().unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn format_test_function(name: &str, foreign_name: &str) -> String {
    format!(
        r#"
        #[link(name = "main_test")]
        extern "C" {{ fn {foreign_name}() -> ffi::Any; }}

        let result: Result<_, _>
            = unsafe {{ _pen_test_convert_result({foreign_name}()) }}.into_result();
        println!("\t{{}}\t{name}", if result.is_ok() {{ "OK" }} else {{ "FAIL" }});

        if let Err(message) = &result {{
            println!("\t\tMessage: {{}}", message);
            error += 1;
        }} else {{
            success += 1;
        }}
        "#,
        name = name,
        foreign_name = foreign_name,
    )
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
