use crate::{context::CompileContext, CompileError};
use hir::{
    analysis::{type_canonicalizer, type_difference_calculator, union_type_creator, AnalysisError},
    ir::*,
    types,
    types::Type,
};
use position::Position;

const ARGUMENT_NAME: &str = "$arg";

struct IfTypeContext<'a> {
    parent: &'a CompileContext,
    name: &'a str,
    argument: Expression,
    position: &'a Position,
}

// This transformation is to reduce code sizes with union type branches
// preventing linear bloat in MIR where union type matching cannot be
// represented natively.
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
            let type_ = type_canonicalizer::canonicalize(&type_, context.types())?;

            Let::new(
                Some(ARGUMENT_NAME.into()),
                Some(type_.clone()),
                if_.argument().clone(),
                compile_partial(
                    &IfTypeContext {
                        parent: context,
                        name: if_.name().into(),
                        argument: Variable::new(ARGUMENT_NAME, position.clone()).into(),
                        position,
                    },
                    &type_,
                    if_.branches(),
                    if_.else_(),
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
    context: &IfTypeContext,
    rest_type: &Type,
    branches: &[IfTypeBranch],
    else_: Option<&ElseBranch>,
    intermediate_branches: Vec<IfTypeBranch>,
) -> Result<Expression, CompileError> {
    let name = context.name;
    let argument = context.argument;
    let position = context.position;

    Ok(match branches {
        [branch, ..] => {
            let branches = &branches[1..];
            let type_ = type_canonicalizer::canonicalize(branch.type_(), context.parent.types())?;

            match type_ {
                Type::Any(_) => return Err(AnalysisError::AnyTypeBranch(position.clone()).into()),
                Type::Union(union) => IfType::new(
                    context.name,
                    context.argument.clone(),
                    intermediate_branches,
                    Some(ElseBranch::new(
                        None,
                        compile_union_branch(context, &union, branch.expression(), else_)?,
                        position.clone(),
                    )),
                    position.clone(),
                )
                .into(),
                _ => compile_partial(
                    context,
                    &rest_type,
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
                    context.parent.types(),
                )?;

                match type_ {
                    Type::Any(_) => {
                        return Err(AnalysisError::AnyTypeBranch(branch.position().clone()).into())
                    }
                    Type::Union(union) => IfType::new(
                        argument.clone(),
                        intermediate_branches,
                        Some(mir::ir::DefaultAlternative::new(
                            name,
                            compile_union_branch(
                                context,
                                &union,
                                &type_difference_calculator::calculate(
                                    rest_type,
                                    &type_,
                                    context.parent.types(),
                                )?
                                .unwrap(),
                                branch.expression(),
                                todo!(),
                            )?,
                        )),
                        position.clone(),
                    )
                    .into(),
                    _ => IfType::new(
                        name,
                        argument.clone(),
                        intermediate_branches
                            .into_iter()
                            .chain([IfTypeBranch::new(type_, branch.expression().clone())])
                            .collect(),
                        None,
                        position.clone(),
                    )
                    .into(),
                }
            } else {
                IfType::new(
                    name,
                    argument.clone(),
                    intermediate_branches,
                    None,
                    position.clone(),
                )
                .into()
            }
        }
    })
}

fn compile_union_branch(
    context: &IfTypeContext,
    type_: &types::Union,
    rest_type: &Type,
    then: &Expression,
    else_: Option<&Expression>,
) -> Result<Expression, CompileError> {
    let argument = context.argument;
    let position = context.position;

    Ok(If::new(
        IfType::new(
            context.name,
            argument.clone(),
            vec![IfTypeBranch::new(type_.clone(), Boolean::new(true, position.clone())).into()],
            Some(ElseBranch::new(
                Some(rest_type.clone()),
                Boolean::new(false, position.clone()),
                position.clone(),
            )),
            position.clone(),
        ),
        then.clone(),
        else_.cloned().unwrap_or_else(|| {
            // Unreachable code
            IfType::new("", argument.clone(), vec![], None, position.clone()).into()
        }),
        position.clone(),
    )
    .into())
}
