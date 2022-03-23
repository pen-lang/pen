use std::io::{stdin, stdout, Read, Write};

const STDIN_PATH: &str = "foo";

pub fn format() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::new();

    stdin().read_to_string(&mut source)?;

    write!(
        stdout(),
        "{}",
        app::module_formatter::format(&source, STDIN_PATH)?
    )?;

    Ok(())
}
