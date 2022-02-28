use crate::{context::CompileContext, CompileError};
use hir::{
    analysis::{ir::expression_visitor, types::type_subsumption_checker},
    ir::*,
    types,
};

pub fn validate(module: &Module, context: &CompileContext) -> Result<(), CompileError> {
    for expression in collect_expressions(module) {
        if let Expression::Let(let_) = expression {
            let expression = let_.bound_expression();
            let type_ = let_
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(expression.position().clone()))?;

            if let_.name().is_none()
                && type_subsumption_checker::check(
                    &types::Reference::new(
                        &context.configuration()?.error_type.error_type_name,
                        type_.position().clone(),
                    )
                    .into(),
                    type_,
                    context.types(),
                )?
            {
                return Err(CompileError::MissingTryOperation(
                    expression.position().clone(),
                ));
            }
        }
    }

    Ok(())
}

fn collect_expressions(module: &Module) -> Vec<Expression> {
    let mut expressions = vec![];

    expression_visitor::visit(module, |expression| {
        if matches!(expression, Expression::Let(_)) {
            expressions.push(expression.clone())
        }
    });

    expressions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{
        test::{DefinitionFake, ModuleFake, TypeDefinitionFake},
        types::{self, Type},
    };
    use once_cell::sync::Lazy;
    use position::{test::PositionFake, Position};

    static ERROR_TYPE: Lazy<Type> = Lazy::new(|| {
        types::Reference::new(
            &COMPILE_CONFIGURATION.error_type.error_type_name,
            Position::fake(),
        )
        .into()
    });
    static ERROR_TYPE_DEFINITION: Lazy<TypeDefinition> = Lazy::new(|| {
        TypeDefinition::fake(
            &COMPILE_CONFIGURATION.error_type.error_type_name,
            vec![],
            false,
            false,
            false,
        )
    });

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            module,
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
        )
    }

    #[test]
    fn fail_to_validate_none_type() {
        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![ERROR_TYPE_DEFINITION.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Let::new(
                                None,
                                Some(types::None::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            ),
            Err(CompileError::MissingTryOperation(Position::fake())),
        );
    }

    #[test]
    fn fail_to_validate_error_type() {
        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![ERROR_TYPE_DEFINITION.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", ERROR_TYPE.clone())],
                            types::None::new(Position::fake()),
                            Let::new(
                                None,
                                Some(ERROR_TYPE.clone()),
                                Variable::new("x", Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            ),
            Err(CompileError::MissingTryOperation(Position::fake())),
        );
    }

    #[test]
    fn fail_to_validate_result_type() {
        let result_type = types::Union::new(
            types::None::new(Position::fake()),
            ERROR_TYPE.clone(),
            Position::fake(),
        );

        assert_eq!(
            validate_module(
                &Module::empty()
                    .set_type_definitions(vec![ERROR_TYPE_DEFINITION.clone()])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", result_type.clone())],
                            types::None::new(Position::fake()),
                            Let::new(
                                None,
                                Some(result_type.into()),
                                Variable::new("x", Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            ),
            Err(CompileError::MissingTryOperation(Position::fake())),
        );
    }
}
