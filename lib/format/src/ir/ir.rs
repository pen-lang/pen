use std::rc::Rc;

// https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf
//
// We need soft-line and if-break nodes to make nodes totally agnostic about if
// parent nodes are broken or not.
// (e.g. handling trailing commas in function calls)

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Flatten(Rc<Document>),
    HardLine,
    Indent(Rc<Document>),
    Line,
    LineSuffix(String),
    Sequence(Rc<[Document]>),
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
        Self::Sequence(documents.into())
    }
}
