use crate::{CompileError, context::Context, expression, type_};
use hir::{
    analysis::{AnalysisError, type_canonicalizer},
    ir::*,
    types::{self, Type},
};

pub fn compile(
    context: &Context,
    comprehension: &ListComprehension,
) -> Result<mir::ir::Expression, CompileError> {
    let branch = &comprehension.branches()[0];

    compile_lists(
        context,
        comprehension.type_(),
        if comprehension.branches().len() == 1 {
            ListElement::Single(comprehension.element().clone())
        } else {
            ListElement::Multiple(
                ListComprehension::new(
                    comprehension.type_().clone(),
                    comprehension.element().clone(),
                    comprehension.branches()[1..].to_vec(),
                    comprehension.position().clone(),
                )
                .into(),
            )
        },
        branch,
    )
}

fn compile_lists(
    context: &Context,
    element_type: &Type,
    element: ListElement,
    branch: &ListComprehensionBranch,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";

    let list_type = type_::compile_list(context)?;
    let definition = compile_function_definition(context, element_type, element, branch)?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![
            mir::ir::LetRecursive::new(
                mir::ir::FunctionDefinition::new(
                    CLOSURE_NAME,
                    vec![],
                    list_type.clone(),
                    mir::ir::LetRecursive::new(
                        definition.clone(),
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                branch
                                    .iteratees()
                                    .iter()
                                    .map(|_| list_type.clone().into())
                                    .collect(),
                                list_type,
                            ),
                            mir::ir::Variable::new(definition.name()),
                            branch
                                .iteratees()
                                .iter()
                                .map(|iteratee| expression::compile(context, iteratee.expression()))
                                .collect::<Result<_, _>>()?,
                        ),
                    ),
                ),
                mir::ir::Variable::new(CLOSURE_NAME),
            )
            .into(),
        ],
    )
    .into())
}

fn compile_function_definition(
    context: &Context,
    element_type: &Type,
    element: ListElement,
    branch: &ListComprehensionBranch,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const LIST_NAME: &str = "$list";

    let position = branch.position();
    let list_type = type_::compile_list(context)?;
    let iteratee_names = (0..branch.iteratees().len())
        .map(|index| format!("{LIST_NAME}_{index}"))
        .collect::<Vec<_>>();
    let iteratee_types = branch
        .iteratees()
        .iter()
        .map(|iteratee| {
            type_canonicalizer::canonicalize_list(
                iteratee
                    .type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(iteratee.position().clone()))?,
                context.types(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut body = {
        let rest = Call::new(
            Some(
                types::Function::new(
                    iteratee_types
                        .iter()
                        .map(|type_| type_.clone().into())
                        .collect(),
                    types::List::new(element_type.clone(), position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(CLOSURE_NAME, position.clone()),
            iteratee_names
                .iter()
                .map(|name| Variable::new(name, position.clone()).into())
                .collect(),
            position.clone(),
        );
        let list = List::new(
            element_type.clone(),
            vec![element, ListElement::Multiple(rest.clone().into())],
            position.clone(),
        );

        if let Some(condition) = branch.condition() {
            Expression::from(If::new(
                condition.clone(),
                list,
                rest,
                branch.position().clone(),
            ))
        } else {
            list.into()
        }
    };

    for ((element_name, iteratee_name), type_) in branch
        .names()
        .iter()
        .zip(&iteratee_names)
        .zip(iteratee_types)
        .rev()
    {
        body = IfList::new(
            Some(type_.element().clone()),
            Variable::new(iteratee_name, position.clone()),
            element_name,
            iteratee_name,
            body,
            List::new(element_type.clone(), vec![], position.clone()),
            position.clone(),
        )
        .into()
    }

    Ok(mir::ir::FunctionDefinition::new(
        CLOSURE_NAME,
        iteratee_names
            .iter()
            .map(|name| mir::ir::Argument::new(name, list_type.clone()))
            .collect(),
        list_type,
        expression::compile(context, &body)?,
    ))
}
