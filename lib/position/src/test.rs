use crate::Position;

pub trait PositionFake {
    fn fake() -> Self;
}

impl PositionFake for Position {
    fn fake() -> Self {
        Self::new("", 0, 0, "")
    }
}
