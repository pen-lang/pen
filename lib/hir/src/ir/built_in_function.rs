use position::Position;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInFunctionName {
    Debug,
    Delete,
    Error,
    Keys,
    Race,
    ReflectDebug,
    ReflectEqual,
    Size,
    Source,
    Spawn,
    Values,
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
