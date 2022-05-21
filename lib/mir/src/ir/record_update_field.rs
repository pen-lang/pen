use super::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdateField {
    index: usize,
    expression: Expression,
}

impl RecordUpdateField {
    pub fn new(index: usize, record: impl Into<Expression>) -> Self {
        Self {
            index,
            expression: record.into(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
