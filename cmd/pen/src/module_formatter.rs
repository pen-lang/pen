use std::{
    error::Error,
    io::{stdin, stdout, Read, Write},
};

const STDIN_PATH: &str = "<stdin>";

pub fn format() -> Result<(), Box<dyn Error>> {
    let mut source = String::new();

    stdin().read_to_string(&mut source)?;

    write!(
        stdout(),
        "{}",
        app::module_formatter::format(&source, STDIN_PATH)?
    )?;

    Ok(())
}
