use crate::context::Context;
use fnv::FnvHashSet;
use hir::{
    analysis::{expression_visitor, union_type_member_calculator, AnalysisError},
    ir::*,
    types::{self, Type},
};

/// Collects types potentially casted from or to variant (union or `any`) types.
pub fn collect(context: &Context, module: &Module) -> Result<FnvHashSet<Type>, AnalysisError> {
    let mut lower_types = FnvHashSet::default();

    // We need to visit expressions other than type coercion too because type
    // coercion might be generated just before compilation.
    //
    // TODO Do such transformation ahead, check types, and collect generic types
    // only from type coercion.
    // https://github.com/pen-lang/pen/issues/1271
    expression_visitor::visit(module, |expression| match expression {
        Expression::Call(call) => {
            if let Expression::BuiltInFunction(function) = call.function() {
                if function.name() == BuiltInFunctionName::Race {
                    let position = call.position();

                    lower_types.insert(
                        types::List::new(types::Any::new(position.clone()), position.clone())
                            .into(),
                    );
                }
            }
        }
        Expression::IfList(if_) => {
            lower_types.insert(if_.type_().unwrap().clone());
        }
        Expression::IfMap(if_) => {
            lower_types.insert(if_.key_type().unwrap().clone());
            lower_types.insert(if_.value_type().unwrap().clone());
        }
        Expression::IfType(if_) => {
            lower_types.extend(
                if_.branches()
                    .iter()
                    .map(|branch| branch.type_())
                    .chain(if_.else_().and_then(|branch| branch.type_()))
                    .cloned(),
            );
        }
        Expression::List(list) => {
            lower_types.insert(list.type_().clone());
        }
        Expression::ListComprehension(comprehension) => {
            for branch in comprehension.branches() {
                if let Some(type_) = branch.type_() {
                    match type_ {
                        Type::List(list_type) => {
                            lower_types.insert(list_type.element().clone());
                        }
                        Type::Map(map_type) => {
                            lower_types.insert(map_type.key().clone());
                            lower_types.insert(map_type.value().clone());
                        }
                        _ => {}
                    }
                }
            }

            lower_types.insert(comprehension.type_().clone());
        }
        Expression::Map(map) => {
            lower_types.insert(map.key_type().clone());
            lower_types.insert(map.value_type().clone());
        }
        Expression::TypeCoercion(coercion) => {
            lower_types.insert(coercion.from().clone());
        }
        Expression::Operation(operation) => match operation {
            Operation::Equality(operation) => {
                lower_types.extend(operation.type_().cloned());
            }
            Operation::Try(operation) => {
                lower_types.extend(operation.type_().cloned());
            }
            _ => {}
        },
        _ => {}
    });

    Ok(lower_types
        .into_iter()
        .chain(
            // Collect types from record fields for type information generation.
            module
                .type_definitions()
                .iter()
                .flat_map(|definition| definition.fields())
                .map(|field| field.type_().clone()),
        )
        .map(|type_| union_type_member_calculator::calculate(&type_, context.types()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::{FunctionDefinitionFake, ModuleFake, TypeDefinitionFake};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn collect_module(module: &Module) -> FnvHashSet<Type> {
        collect(
            &Context::new(module, Some(COMPILE_CONFIGURATION.clone())),
            module,
        )
        .unwrap()
    }

    #[test]
    fn collect_nothing() {
        assert_eq!(collect_module(&Module::empty()), [].into_iter().collect());
    }

    #[test]
    fn collect_from_equal_operation() {
        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(
                vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::Number::new(Position::fake()),
                                    Position::fake()
                                )
                                .into()
                            ),
                            EqualityOperator::Equal,
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]
            )),
            [
                types::None::new(Position::fake()).into(),
                types::Number::new(Position::fake()).into()
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn collect_from_list() {
        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::List::new(types::None::new(Position::fake()), Position::fake()),
                        List::new(types::None::new(Position::fake()), vec![], Position::fake(),),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            [types::None::new(Position::fake()).into()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn collect_from_map() {
        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Map::new(
                            types::None::new(Position::fake()),
                            types::Number::new(Position::fake()),
                            Position::fake()
                        ),
                        Map::new(
                            types::None::new(Position::fake()),
                            types::Number::new(Position::fake()),
                            vec![],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            [
                types::None::new(Position::fake()).into(),
                types::Number::new(Position::fake()).into()
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn collect_from_race_function_call() {
        let list_type = types::List::new(
            types::List::new(types::None::new(Position::fake()), Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(
                vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            list_type.element().clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![list_type.clone().into()],
                                        list_type.element().clone(),
                                        Position::fake(),
                                    )
                                    .into(),
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Race, Position::fake()),
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]
            )),
            [types::List::new(types::Any::new(Position::fake()), Position::fake()).into()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn collect_from_try_operation() {
        let list_type = types::List::new(
            types::List::new(types::None::new(Position::fake()), Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::Boolean::new(Position::fake()),
                        TryOperation::new(
                            Some(types::None::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            [types::None::new(Position::fake()).into(),]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn collect_from_type_coercion() {
        assert_eq!(
            collect_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Any::new(Position::fake()),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            types::Any::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            [types::None::new(Position::fake()).into()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn collect_from_record_field() {
        let field_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            collect_module(
                &Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                    "r",
                    vec![types::RecordField::new("x", field_type.clone())],
                    false,
                    false,
                    false,
                )])
            ),
            [field_type.into()].into_iter().collect()
        );
    }

    mod list_comprehension {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn collect_from_list() {
            let input_list_type =
                types::List::new(types::Number::new(Position::fake()), Position::fake());
            let output_list_type =
                types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                collect_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", input_list_type.clone())],
                            output_list_type.clone(),
                            ListComprehension::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake(),),
                                vec![ListComprehensionBranch::new(
                                    Some(input_list_type.clone().into()),
                                    "x",
                                    None,
                                    Variable::new("x", Position::fake()),
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    ),
                ])),
                [
                    input_list_type.element().clone(),
                    output_list_type.element().clone()
                ]
                .into_iter()
                .collect()
            );
        }

        #[test]
        fn collect_from_map() {
            let input_map_type = types::Map::new(
                types::ByteString::new(Position::fake()),
                types::Number::new(Position::fake()),
                Position::fake(),
            );
            let output_list_type =
                types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                collect_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", input_map_type.clone())],
                            output_list_type.clone(),
                            ListComprehension::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake(),),
                                vec![ListComprehensionBranch::new(
                                    Some(input_map_type.clone().into()),
                                    "k",
                                    Some("v".into()),
                                    Variable::new("x", Position::fake()),
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    ),
                ])),
                [
                    input_map_type.key().clone(),
                    input_map_type.value().clone(),
                    output_list_type.element().clone()
                ]
                .into_iter()
                .collect()
            );
        }
    }
}
