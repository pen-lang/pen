#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberRepresentation {
    Binary(String),
    Hexadecimal(String),
    FloatingPoint(String),
}
