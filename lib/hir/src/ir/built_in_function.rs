#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInFunction {
    Debug,
    Error,
    Size,
    Source,
    Spawn,
}
