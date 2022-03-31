// https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Flatten(Box<Document>),
    HardLine,
    Indent(Box<Document>),
    Line,
    LineSuffix(String),
    Sequence(Vec<Document>),
    SoftLine,
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
