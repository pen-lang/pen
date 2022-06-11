use std::rc::Rc;

// https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf
//
// Unlike the Wadler's algorithm or some other formatters like prettier, we do
// not need to search the best format given source codes. For example, we do
// not have any "group" combinator.
//
// However, we are rather given the "best" format by all information available
// in the source codes like Go.
//
// We need soft-line and if-break nodes to make nodes totally agnostic about if
// parent nodes are broken or not. But that also makes IR more complex.
// (e.g. handling trailing commas in function calls)

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Break(bool, Rc<Document>),
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

impl From<Vec<Self>> for Document {
    fn from(documents: Vec<Self>) -> Self {
        Self::Sequence(documents.into())
    }
}
