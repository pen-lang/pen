use super::type_::Type;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Function {
    argument: Arc<Type>,
    result: Arc<Type>,
}

impl Function {
    pub fn new(argument: impl Into<Type>, result: impl Into<Type>) -> Self {
        Self {
            argument: argument.into().into(),
            result: result.into().into(),
        }
    }

    pub fn argument(&self) -> &Type {
        &self.argument
    }

    pub fn arguments(&self) -> impl IntoIterator<Item = &Type> {
        let mut arguments = vec![self.argument()];
        let mut type_ = self;

        while let Type::Function(function) = type_.result() {
            arguments.push(function.argument());
            type_ = function;
        }

        arguments
    }

    pub fn result(&self) -> &Type {
        &self.result
    }

    pub fn last_result(&self) -> &Type {
        let mut type_ = self;

        while let Type::Function(function) = type_.result() {
            type_ = function;
        }

        type_.result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument() {
        assert_eq!(
            Function::new(Type::Number, Type::Number).argument(),
            &Type::Number
        );
    }

    #[test]
    fn result() {
        assert_eq!(
            Function::new(Type::Number, Type::Number).result(),
            &Type::Number
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(
            Function::new(Type::Number, Type::Number,)
                .arguments()
                .into_iter()
                .collect::<Vec<&Type>>(),
            vec![&Type::Number]
        );

        assert_eq!(
            Function::new(Type::Number, Function::new(Type::Number, Type::Number))
                .arguments()
                .into_iter()
                .collect::<Vec<&Type>>(),
            vec![&Type::Number, &Type::Number]
        );
    }

    #[test]
    fn last_result() {
        assert_eq!(
            Function::new(Type::Number, Function::new(Type::Number, Type::Number)).last_result(),
            &Type::Number
        );
    }
}
