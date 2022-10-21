use super::expression::Expression;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Rc<Expression>,
    then: Rc<Expression>,
    else_: Rc<Expression>,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
    ) -> Self {
        Self {
            condition: Rc::new(condition.into()),
            then: Rc::new(then.into()),
            else_: Rc::new(else_.into()),
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn then(&self) -> &Expression {
        &self.then
    }

    pub fn else_(&self) -> &Expression {
        &self.else_
    }
}
