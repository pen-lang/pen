use crate::{analysis::expression_conversion, ir::*};

pub fn transform(module: &Module) -> Module {
    expression_conversion::transform(module, |expression| match expression {
        Expression::StringConcatenation(concatenation) => {
            let mut operands = vec![];

            for operand in concatenation.operands() {
                if let Expression::StringConcatenation(concatenation) = operand {
                    operands.extend(concatenation.operands().iter().cloned());
                } else {
                    operands.push(operand.clone());
                }
            }

            StringConcatenation::new(operands).into()
        }
        _ => expression.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test::ModuleFake, types::Type};

    #[test]
    fn transform_concatenation() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "f",
            vec![],
            Type::ByteString,
            StringConcatenation::new(vec![]),
        )]);

        assert_eq!(transform(&module), module);
    }

    #[test]
    fn transform_nested_concatenation() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::ByteString,
                    StringConcatenation::new(vec![StringConcatenation::new(vec![]).into()]),
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::ByteString,
                StringConcatenation::new(vec![]),
            )])
        );
    }

    #[test]
    fn transform_nested_concatenation_with_strings() {
        assert_eq!(
            transform(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    Type::ByteString,
                    StringConcatenation::new(vec![
                        ByteString::new("foo").into(),
                        StringConcatenation::new(vec![ByteString::new("bar").into()]).into(),
                        ByteString::new("baz").into()
                    ]),
                )])
            ),
            Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::ByteString,
                StringConcatenation::new(vec![
                    ByteString::new("foo").into(),
                    ByteString::new("bar").into(),
                    ByteString::new("baz").into(),
                ]),
            )])
        );
    }
}
