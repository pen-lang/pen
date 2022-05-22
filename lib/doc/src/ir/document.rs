#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub title: Text,
    pub paragraphs: Vec<Paragraph>,
    pub children: Vec<Section>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Paragraph {
    Text(Text),
    Code { language: String, code: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub spans: Vec<Span>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    Normal(String),
    Code(String),
}

impl From<Text> for Paragraph {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}
