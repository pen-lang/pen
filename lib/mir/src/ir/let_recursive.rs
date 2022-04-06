use super::{definition::FunctionDefinition, expression::Expression};
use std::sync::Arc;

// Function definitions in let-recursive expressions are recursive for the
// following reasons although we don't have any recursive function
// definitions in practice.
//
// - Variants of typed lambda calculus used in other languages like OCaml are
//   also implemented like this.
// - In reference counting, we need to access closure objects inside their
//   functions themselves to drop them properly because they are moved into
//   function calls.
//   - This requirement can be potentially removed by dropping every functions
//     used in function calls inside the functions themselves.
//     - This is safe because only global function definitions can be recursive.
// - This recursion is necessary to compile "anonymous" loops in HIR
//   effectively.
//   - e.g. list comprehension
#[derive(Clone, Debug, PartialEq)]
pub struct LetRecursive {
    definition: Arc<FunctionDefinition>,
    expression: Arc<Expression>,
}

impl LetRecursive {
    pub fn new(definition: FunctionDefinition, expression: impl Into<Expression>) -> Self {
        Self {
            definition: definition.into(),
            expression: Arc::new(expression.into()),
        }
    }

    pub fn definition(&self) -> &FunctionDefinition {
        &self.definition
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
