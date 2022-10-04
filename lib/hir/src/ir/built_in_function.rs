use position::Position;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInFunctionName {
    Debug,
    Error,
    Race,
    ReflectDebug,
    Size,
    Source,
    Spawn,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BuiltInFunction {
    name: BuiltInFunctionName,
    position: Position,
}

impl BuiltInFunction {
    pub fn new(name: BuiltInFunctionName, position: Position) -> Self {
        Self { name, position }
    }

    pub fn name(&self) -> BuiltInFunctionName {
        self.name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
