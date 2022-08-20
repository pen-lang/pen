use crate::{context::CompileContext, CompileError};
use fnv::FnvHashSet;
use hir::analysis::{
    expression_visitor, type_canonicalizer, type_comparability_checker, type_visitor, AnalysisError,
};
use hir::ir::*;
use hir::types::{self, Type};
use position::Position;

pub fn transform_list(context: &CompileContext, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.list_type.list_type_name,
        position.clone(),
    )
    .into())
}

pub fn transform_map(context: &CompileContext, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.map_type_name,
        position.clone(),
    )
    .into())
}

pub fn transform_map_context(
    context: &CompileContext,
    position: &Position,
) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.context_type_name,
        position.clone(),
    )
    .into())
}

pub fn collect_parameter_types(
    context: &CompileContext,
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
