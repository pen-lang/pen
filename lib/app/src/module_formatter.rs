use parse::{parse, parse_comments};
use std::error::Error;

pub fn format(source: &str, path: &str) -> Result<String, Box<dyn Error>> {
    let a = parse(source, path)?;
    let c = parse_comments(source, path)?;

    for _ in 0..10000 {
        format::format(&a, &c);
    }

    Ok(format::format(&a, &c))
}
