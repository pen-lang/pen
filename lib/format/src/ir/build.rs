use super::Document;

pub fn comment(string: impl Into<String>) -> Document {
    Document::Comment(string.into())
}

pub fn group(document: impl Into<Document>, broken: bool) -> Document {
    Document::Group {
        document: document.into().into(),
        broken,
    }
}

pub fn line(document: impl Into<Document>, soft: bool) -> Document {
    Document::Line {
        document: document.into().into(),
        soft,
    }
}

pub fn indent(document: impl Into<Document>) -> Document {
    Document::Indent(document.into().into())
}
