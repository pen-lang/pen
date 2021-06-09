use super::expression::Expression;
use crate::types::{self, Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionApplication {
    type_: types::Function,
    function: Arc<Expression>,
    argument: Arc<Expression>,
}

impl FunctionApplication {
    pub fn new(
        type_: types::Function,
        function: impl Into<Expression>,
        argument: impl Into<Expression>,
    ) -> Self {
        Self {
            type_,
            function: function.into().into(),
            argument: argument.into().into(),
        }
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn first_function(&self) -> &Expression {
        let mut function: &Expression = &self.function;

        while let Expression::FunctionApplication(function_application) = function {
            function = function_application.function();
        }

        function
    }

    pub fn arguments(&self) -> impl IntoIterator<Item = &Expression> {
        let mut arguments = vec![self.argument()];
        let mut expression = self;

        while let Expression::FunctionApplication(function_application) = expression.function() {
            arguments.push(function_application.argument());
            expression = function_application;
        }

        arguments.reverse();

        arguments
    }

    pub fn argument_types(&self) -> impl IntoIterator<Item = &Type> {
        if let Expression::FunctionApplication(application) = self.function.as_ref() {
            application.argument_types().into_iter().collect::<Vec<_>>()
        } else {
            self.type_().arguments().into_iter().collect::<Vec<_>>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::variable::Variable, *};
    use crate::types::Type;
    use once_cell::sync::Lazy;

    static FAKE_FUNCTION_TYPE: Lazy<types::Function> =
        Lazy::new(|| types::Function::new(Type::Number, Type::Number));

    #[test]
    fn first_function() {
        assert_eq!(
            FunctionApplication::new(FAKE_FUNCTION_TYPE.clone(), Variable::new("f"), 42.0)
                .first_function(),
            &Variable::new("f").into()
        );

        assert_eq!(
            FunctionApplication::new(
                FAKE_FUNCTION_TYPE.clone(),
                FunctionApplication::new(FAKE_FUNCTION_TYPE.clone(), Variable::new("f"), 1.0),
                2.0
            )
            .first_function(),
            &Variable::new("f").into()
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(
            FunctionApplication::new(FAKE_FUNCTION_TYPE.clone(), Variable::new("f"), 42.0)
                .arguments()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![42.0.into()]
        );

        assert_eq!(
            FunctionApplication::new(
                FAKE_FUNCTION_TYPE.clone(),
                FunctionApplication::new(FAKE_FUNCTION_TYPE.clone(), Variable::new("f"), 1.0),
                2.0
            )
            .arguments()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
            vec![1.0.into(), 2.0.into()]
        );
    }
}
