use super::{Definition, Expression};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Definition(Definition),
}
