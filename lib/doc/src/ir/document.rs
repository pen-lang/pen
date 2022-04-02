pub struct Section {
    pub title: Text,
    pub paragraphs: Vec<Paragraph>,
    pub children: Vec<Section>,
}

pub enum Paragraph {
    Text(Text),
    Code(String),
}

pub struct Text {
    pub spans: Vec<Span>,
}

pub enum Span {
    Normal(String),
    Code(String),
}

impl From<Text> for Paragraph {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}
