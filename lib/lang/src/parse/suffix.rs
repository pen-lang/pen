use crate::ast::*;

#[derive(Clone, Debug)]
pub enum SuffixOperator {
    Call(Vec<Expression>),
    Element(String),
}
