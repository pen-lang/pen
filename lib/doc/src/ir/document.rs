pub struct Section {
    pub title: Text,
    pub paragraphs: Vec<Text>,
    pub children: Vec<Section>,
}

pub struct Text {
    pub spans: Vec<Span>,
}

pub enum Span {
    Normal(String),
    Code(String),
}
