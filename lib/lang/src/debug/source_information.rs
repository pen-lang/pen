use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::Display,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SourceInformation {
    path: String,
    line_number: usize,
    column_number: usize,
    line: String,
}

impl SourceInformation {
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
}

impl Display for SourceInformation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let line_information = format!("{}:{}:", self.line_number, self.column_number);

        write!(
            formatter,
            "{}\n{}\t{}\n{}\t{}^",
            self.path,
            &line_information,
            self.line,
            str::repeat(" ", line_information.len()),
            str::repeat(" ", self.column_number - 1),
        )
    }
}

impl Eq for SourceInformation {}

impl PartialEq for SourceInformation {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Ord for SourceInformation {
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl PartialOrd for SourceInformation {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl Hash for SourceInformation {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

#[cfg(test)]
mod tests {
    use super::SourceInformation;

    #[test]
    fn display() {
        assert_eq!(
            format!("{}", SourceInformation::new("file", 1, 1, "x")),
            "file\n1:1:\tx\n    \t^"
        );

        assert_eq!(
            format!("{}", SourceInformation::new("file", 1, 2, " x")),
            "file\n1:2:\t x\n    \t ^"
        );
    }
}
