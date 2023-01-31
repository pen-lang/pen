use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn log_error(error: &dyn std::error::Error) -> Result<(), Box<dyn std::error::Error>> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    write!(stderr, "error")?;
    stderr.set_color(ColorSpec::new().set_fg(None))?;

    writeln!(
        stderr,
        ": {}",
        format!("{error}").replace('\n', "\n  ").trim()
    )?;

    if let Some(error) = error.source() {
        log_error(error)?;
    }

    Ok(())
}

pub fn log_info(log: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    write!(stderr, "info")?;
    stderr.set_color(ColorSpec::new().set_fg(None))?;

    writeln!(stderr, ": {log}")?;

    Ok(())
}
