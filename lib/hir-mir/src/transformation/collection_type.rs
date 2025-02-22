use crate::{CompileError, context::Context};
use fnv::FnvHashSet;
use hir::{
    analysis::{
        AnalysisError, expression_visitor, type_canonicalizer, type_comparability_checker,
        type_visitor,
    },
    ir::*,
    types::{self, Type},
};
use position::Position;

pub fn transform_list(context: &Context, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.list_type.list_type_name,
        position.clone(),
    )
    .into())
}

pub fn transform_map(context: &Context, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.map_type_name,
        position.clone(),
    )
    .into())
}

pub fn transform_map_context(context: &Context, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.context_type_name,
        position.clone(),
    )
    .into())
}

pub fn collect_comparable_parameter_types(
    context: &Context,
    module: &Module,
) -> Result<FnvHashSet<Type>, AnalysisError> {
    let mut types = FnvHashSet::default();

    type_visitor::visit(module, |type_| match type_ {
        Type::List(list_type) => {
            types.insert(list_type.element());
        }
        Type::Map(map_type) => {
            types.extend([map_type.key(), map_type.value()]);
        }
        _ => {}
    });

    expression_visitor::visit(module, |expression| match expression {
        Expression::IfList(if_) => {
            types.extend(if_.type_());
        }
        Expression::IfMap(if_) => {
            types.extend([if_.key_type(), if_.value_type()].into_iter().flatten());
        }
        Expression::List(list) => {
            types.insert(list.type_());
        }
        Expression::ListComprehension(comprehension) => {
            types.insert(comprehension.type_());
        }
        Expression::Map(map) => {
            types.extend([map.key_type(), map.value_type()]);
        }
        _ => {}
    });

    Ok(types
        .iter()
        .map(|type_| {
            let type_ = type_canonicalizer::canonicalize(type_, context.types())?;

            Ok(
                if type_comparability_checker::check(&type_, context.types(), context.records())? {
                    Some(type_)
                } else {
                    None
                },
            )
        })
        .collect::<Result<FnvHashSet<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::test::{ModuleFake, TypeAliasFake};
    use position::test::PositionFake;

    mod comparable_parameter_type_collection {
        use super::*;

        fn collect(module: &Module) -> Result<FnvHashSet<Type>, AnalysisError> {
            collect_comparable_parameter_types(
                &Context::new(module, Some(COMPILE_CONFIGURATION.clone())),
                module,
            )
        }

        #[test]
        fn collect_no_type() {
            assert_eq!(collect(&Module::empty()), Ok(Default::default()));
        }

        #[test]
        fn collect_type() {
            assert_eq!(
                collect(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                    "a",
                    types::List::new(types::None::new(Position::fake()), Position::fake()),
                    false,
                    false,
                )])),
                Ok([types::None::new(Position::fake()).into()]
                    .into_iter()
                    .collect())
            );
        }

        #[test]
        fn collect_types() {
            assert_eq!(
                collect(&Module::empty().set_type_aliases(vec![
                    TypeAlias::fake(
                        "a",
                        types::List::new(types::None::new(Position::fake()), Position::fake()),
                        false,
                        false,
                    ),
                    TypeAlias::fake(
                        "b",
                        types::List::new(types::Number::new(Position::fake()), Position::fake()),
                        false,
                        false,
                    )
                ])),
                Ok([
                    types::None::new(Position::fake()).into(),
                    types::Number::new(Position::fake()).into()
                ]
                .into_iter()
                .collect())
            );
        }

        #[test]
        fn collect_duplicate_types() {
            assert_eq!(
                collect(&Module::empty().set_type_aliases(vec![
                    TypeAlias::fake(
                        "a",
                        types::List::new(types::None::new(Position::fake()), Position::fake()),
                        false,
                        false,
                    ),
                    TypeAlias::fake("b", types::None::new(Position::fake()), false, false,),
                    TypeAlias::fake(
                        "c",
                        types::List::new(
                            types::Reference::new("b", Position::fake()),
                            Position::fake()
                        ),
                        false,
                        false,
                    )
                ])),
                Ok([types::None::new(Position::fake()).into()]
                    .into_iter()
                    .collect())
            );
        }

        #[test]
        fn collect_no_any_type() {
            assert_eq!(
                collect(&Module::empty().set_type_aliases(vec![TypeAlias::fake(
                    "a",
                    types::List::new(types::Any::new(Position::fake()), Position::fake()),
                    false,
                    false,
                ),])),
                Ok(Default::default())
            );
        }
    }
}
