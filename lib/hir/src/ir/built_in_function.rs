#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInFunction {
    Debug,
    Error,
    Race,
    Size,
    Source,
    Spawn,
}
