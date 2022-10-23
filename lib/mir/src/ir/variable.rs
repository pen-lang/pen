use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variable {
    name: Rc<str>,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
