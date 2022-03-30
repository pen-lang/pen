use super::Document;

pub fn comment(string: impl Into<String>) -> Document {
    Document::Comment(string.into())
}

pub fn flatten(document: impl Into<Document>) -> Document {
    Document::Flatten(document.into().into())
}

pub fn indent(document: impl Into<Document>) -> Document {
    Document::Indent(document.into().into())
}

pub fn line() -> Document {
    Document::Line
}

pub fn hard_line() -> Document {
    vec![Document::Break, Document::Line].into()
}
