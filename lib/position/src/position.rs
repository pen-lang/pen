use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    path: String,
    line_number: usize,
    column_number: usize,
    line: String,
}

impl Position {
    pub fn new(
        path: impl Into<String>,
        line_number: usize,
        column_number: usize,
        line: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line_number,
            column_number,
            line: line.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let line_information = format!("{}:{}:", self.line_number, self.column_number);

        write!(
            formatter,
            "{}\n{}\t{}\n{}\t{}^",
            self.path,
            &line_information,
            self.line,
            " ".repeat(line_information.len()),
            " ".repeat(self.column_number - 1),
        )
    }
}

impl Eq for Position {}

impl PartialEq for Position {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl Ord for Position {
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn display() {
        assert_eq!(
            format!("{}", Position::new("file", 1, 1, "x")),
            "file\n1:1:\tx\n    \t^"
        );

        assert_eq!(
            format!("{}", Position::new("file", 1, 2, " x")),
            "file\n1:2:\t x\n    \t ^"
        );
    }
}
