pub mod build;
pub mod format;

// https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    // TODO Remove this.
    Comment(String),
    Sequence(Vec<Document>),
    // TODO Remove this?
    Flatten(Box<Document>),
    Line,
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
