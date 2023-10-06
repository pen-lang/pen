use crate::format;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position(Rc<PositionInner>);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PositionInner {
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
        Self(
            PositionInner {
                path: path.into(),
                line_number,
                column_number,
                line: line.into(),
            }
            .into(),
        )
    }

    pub fn path(&self) -> &str {
        &self.0.path
    }

    pub fn line_number(&self) -> usize {
        self.0.line_number
    }

    pub fn column_number(&self) -> usize {
        self.0.column_number
    }

    pub fn line(&self) -> &str {
        &self.0.line
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", format(self))
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
