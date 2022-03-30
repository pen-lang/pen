pub mod build;
pub mod format;

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Comment(String),
    Documents(Vec<Document>),
    Group {
        document: Box<Document>,
        broken: bool,
    },
    Line {
        document: Box<Document>,
        soft: bool,
    },
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
        Self::Documents(documents)
    }
}
