use crate::Position;

pub fn format(position: &Position) -> String {
    format!(
        "{}\n{}\t{}\n{}^",
        position.path(),
        line_information(position),
        position.line(),
        offset(position),
    )
}

pub fn format_message(position: &Position, message: &str) -> String {
    format!("{}\n{}{}", format(position), offset(position), message)
}

fn line_information(position: &Position) -> String {
    format!("{}:{}:", position.line_number(), position.column_number())
}

fn offset(position: &Position) -> String {
    format!(
        "{}\t{}",
        " ".repeat(line_information(position).len()),
        " ".repeat(position.column_number() - 1)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_position() {
        assert_eq!(
            format(&Position::new("file", 1, 1, "x")),
            "file\n1:1:\tx\n    \t^"
        );

        assert_eq!(
            format(&Position::new("file", 1, 2, " x")),
            "file\n1:2:\t x\n    \t ^"
        );
    }

    #[test]
    fn format_position_with_message() {
        assert_eq!(
            format_message(&Position::new("file", 1, 2, "x"), "foo"),
            "file\n1:2:\tx\n    \t ^\n    \t foo"
        );
    }
}
