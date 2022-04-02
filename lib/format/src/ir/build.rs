use super::Document;

pub fn sequence<D: Into<Document>>(iterator: impl IntoIterator<Item = D>) -> Document {
    Document::Sequence(
        iterator
            .into_iter()
            .map(|document| document.into())
            .collect(),
    )
}

pub fn line_suffix(string: impl Into<String>) -> Document {
    Document::LineSuffix(string.into())
}

pub fn flatten(document: impl Into<Document>) -> Document {
    Document::Break(false, document.into().into())
}

pub fn break_(document: impl Into<Document>) -> Document {
    Document::Break(true, document.into().into())
}

pub fn flatten_if(condition: bool, document: impl Into<Document>) -> Document {
    Document::Break(!condition, document.into().into())
}

pub fn indent(document: impl Into<Document>) -> Document {
    Document::Indent(document.into().into())
}

pub const fn line() -> Document {
    Document::Line(false)
}

pub const fn hard_line() -> Document {
    Document::Line(true)
}

pub fn empty() -> Document {
    "".into()
}
