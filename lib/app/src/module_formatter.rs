use parse::{parse, parse_comments};
use std::error::Error;

pub fn format(source: &str, path: &str) -> Result<String, Box<dyn Error>> {
    Ok(format::format(
        &parse(source, path)?,
        &parse_comments(source, path)?,
    ))
}
