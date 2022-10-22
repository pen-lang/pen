use super::expression::Expression;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct If(Rc<IfInner>);

#[derive(Debug, PartialEq)]
struct IfInner {
    condition: Expression,
    then: Expression,
    else_: Expression,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
    ) -> Self {
        Self(
            IfInner {
                condition: (condition.into()),
                then: (then.into()),
                else_: (else_.into()),
            }
            .into(),
        )
    }

    pub fn condition(&self) -> &Expression {
        &self.0.condition
    }

    pub fn then(&self) -> &Expression {
        &self.0.then
    }

    pub fn else_(&self) -> &Expression {
        &self.0.else_
    }
}
