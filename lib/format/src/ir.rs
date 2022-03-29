use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Comment(String),
    Documents(Vec<Document>),
    Group(Group),
    HardBreak,
    Indent(Indent),
    Join(Join),
    SoftBreak,
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    document: Arc<Document>,
    broken: bool,
}

impl Group {
    pub fn new(document: impl Into<Document>, broken: bool) -> Self {
        Self {
            document: document.into().into(),
            broken,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn broken(&self) -> bool {
        self.broken
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Indent {
    document: Arc<Document>,
}

impl Indent {
    pub fn new(document: impl Into<Document>) -> Self {
        Self {
            document: document.into().into(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    documents: Vec<Document>,
    separator: String,
}

impl Join {
    pub fn new(documents: Vec<Document>, separator: impl Into<String>) -> Self {
        Self {
            documents,
            separator: separator.into(),
        }
    }

    pub fn documents(&self) -> &[Document] {
        &self.documents
    }

    pub fn separator(&self) -> &str {
        &self.separator
    }
}
