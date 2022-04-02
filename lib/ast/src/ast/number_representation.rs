#[derive(Clone, Debug, PartialEq)]
pub enum NumberRepresentation {
    Binary(String),
    Hexadecimal(String),
    FloatingPoint(String),
}
