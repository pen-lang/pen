pub mod build;
pub mod format;
mod utils;

pub use utils::*;

// https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    // TODO Remove this.
    Comment(String),
    Sequence(Vec<Document>),
    // TODO Replace this with a function?
    Flatten(Box<Document>),
    HardLine,
    SoftLine,
    Indent(Box<Document>),
    String(String),
}

impl From<&str> for Document {
    fn from(string: &str) -> Self {
        Self::String(string.into())
    }
}

impl From<String> for Document {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<Vec<Document>> for Document {
    fn from(documents: Vec<Document>) -> Self {
        Self::Sequence(documents)
    }
}
