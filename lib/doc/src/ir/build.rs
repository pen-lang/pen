use super::document::*;

pub fn section(
    title: impl Into<Text>,
    paragraphs: impl IntoIterator<Item = Text>,
    children: impl IntoIterator<Item = Section>,
) -> Section {
    Section {
        title: title.into(),
        paragraphs: paragraphs.into_iter().collect(),
        children: children.into_iter().collect(),
    }
}

pub fn text(spans: impl IntoIterator<Item = Span>) -> Text {
    Text {
        spans: spans.into_iter().collect(),
    }
}

pub fn normal(string: impl Into<String>) -> Span {
    Span::Normal(string.into())
}

pub fn code(string: impl Into<String>) -> Span {
    Span::Code(string.into())
}
