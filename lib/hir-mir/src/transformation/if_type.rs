use crate::{context::CompileContext, CompileError};
use hir::analysis::{type_canonicalizer, union_type_creator, AnalysisError};
use hir::ir::*;
use hir::types;
use hir::types::Type;
use position::Position;

const ARGUMENT_NAME: &str = "$arg";

// This transformation is to reduce code sizes with union type branches preventing linear bloat in MIR where union type matching cannot be represented natively.
pub fn compile(context: &CompileContext, if_: &IfType) -> Result<Expression, CompileError> {
    let position = if_.position();

    Ok(
        if let Some(type_) = union_type_creator::create(
            &if_.branches()
                .iter()
                .map(|branch| branch.type_())
                .chain(if_.else_().and_then(|else_| else_.type_()))
                .cloned()
                .collect::<Vec<_>>(),
            position,
        ) {
            Let::new(
                Some(ARGUMENT_NAME.into()),
                Some(type_canonicalizer::canonicalize(&type_, context.types())?),
                if_.argument().clone(),
                compile_partial(
                    context,
                    if_.name(),
                    &Variable::new(ARGUMENT_NAME, position.clone()).into(),
                    if_.branches(),
                    if_.else_(),
                    if_.position(),
                    vec![],
                )?,
                position.clone(),
            )
            .into()
        } else {
            if_.clone().into()
        },
    )
}

fn compile_partial(
    context: &CompileContext,
    name: &str,
    argument: &Expression,
    branches: &[IfTypeBranch],
    else_: Option<&ElseBranch>,
    position: &Position,
    intermediate_branches: Vec<IfTypeBranch>,
) -> Result<Expression, CompileError> {
    Ok(match branches {
        [branch, ..] => {
            let branches = &branches[1..];
            let type_ = type_canonicalizer::canonicalize(branch.type_(), context.types())?;

            match type_ {
                Type::Any(_) => return Err(AnalysisError::AnyTypeBranch(position.clone()).into()),
                Type::Union(union) => IfType::new(
                    name,
                    argument.clone(),
                    intermediate_branches,
                    Some(ElseBranch::new(
                        None,
                        compile_union_branch(
                            context,
                            &union,
                            name,
                            argument,
                            branch.expression(),
                            else_,
                            position,
                        )?,
                        position.clone(),
                    )),
                    position.clone(),
                )
                .into(),
                _ => compile_partial(
                    context,
                    name,
                    argument,
                    branches,
                    else_,
                    position,
                    intermediate_branches
                        .into_iter()
                        .chain([branch.clone()])
                        .collect(),
                )?,
            }
        }
        [] => {
            if let Some(branch) = else_ {
                let type_ = type_canonicalizer::canonicalize(
                    branch
                        .type_()
                        .ok_or_else(|| AnalysisError::TypeNotInferred(branch.position().clone()))?,
                    context.types(),
                )?;

                match type_ {
                    Type::Any(_) => {
                        return Err(AnalysisError::AnyTypeBranch(branch.position().clone()).into())
                    }
                    Type::Union(union) => mir::ir::Case::new(
                        argument.clone(),
                        intermediate_branches,
                        Some(mir::ir::DefaultAlternative::new(
                            name,
                            compile_union_branch(
                                context,
                                &union,
                                name,
                                argument,
                                branch.expression(),
                                else_,
                            )?,
                        )),
                    )
                    .into(),
                    _ => mir::ir::Case::new(
                        argument.clone(),
                        intermediate_branches
                            .into_iter()
                            .chain(compile_alternatives(
                                context,
                                name,
                                &type_,
                                branch.expression(),
                            )?)
                            .collect(),
                        None,
                    )
                    .into(),
                }
            } else {
                IfType::new(
                    name,
                    argument.clone(),
                    intermediate_branches,
                    else_.cloned(),
                    position.clone(),
                )
                .into()
            }
        }
    })
}

fn compile_union_branch(
    context: &CompileContext,
    type_: &types::Union,
    name: &str,
    value: &Expression,
    then: &Expression,
    else_: Option<&ElseBranch>,
    position: &Position,
) -> Result<Expression, CompileError> {
    Ok(If::new(
        IfType::new(
            name,
            value.clone(),
            vec![IfTypeBranch::new(type_.clone(), Boolean::new(true, position.clone())).into()],
            Some(ElseBranch::new(
                todo!(),
                Boolean::new(false, position.clone()),
                position.clone(),
            )),
            position.clone(),
        ),
        then.clone(),
        else_.cloned().unwrap_or_else(|| {
            ElseBranch::new(
                IfType::new(
                    "",
                    None::new(position.clone()),
                    vec![],
                    None,
                    position.clone(),
                ),
                position.clone(),
            )
        }),
        position.clone(),
    )
    .into())
}
