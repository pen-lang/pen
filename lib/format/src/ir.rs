use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    String(String),
    Group(Group),
    Indent(Indent),
    Comment(String),
    HardBreak,
    SoftBreak,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    documents: Vec<Document>,
    broken: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Indent {
    document: Arc<Document>,
}
