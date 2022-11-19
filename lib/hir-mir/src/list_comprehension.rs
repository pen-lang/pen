use crate::{context::Context, expression, type_, CompileError};
use hir::{analysis::AnalysisError, ir::*, types};

pub fn compile(
    context: &Context,
    comprehension: &ListComprehension,
) -> Result<mir::ir::Expression, CompileError> {
    let compile = |expression| expression::compile(context, expression);

    const CLOSURE_NAME: &str = "$loop";
    const LIST_NAME: &str = "$list";

    let position = comprehension.position();
    let input_element_type = comprehension
        .primary_input_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let output_element_type = comprehension.output_type();
    let list_type = type_::compile_list(context)?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                CLOSURE_NAME,
                vec![],
                list_type.clone(),
                mir::ir::LetRecursive::new(
                    mir::ir::FunctionDefinition::new(
                        CLOSURE_NAME,
                        vec![mir::ir::Argument::new(LIST_NAME, list_type.clone())],
                        list_type.clone(),
                        compile(
                            &IfList::new(
                                Some(input_element_type.clone()),
                                Variable::new(LIST_NAME, position.clone()),
                                comprehension.primary_name(),
                                LIST_NAME,
                                List::new(
                                    output_element_type.clone(),
                                    vec![
                                        ListElement::Single(comprehension.element().clone()),
                                        ListElement::Multiple(
                                            Call::new(
                                                Some(
                                                    types::Function::new(
                                                        vec![types::List::new(
                                                            input_element_type.clone(),
                                                            position.clone(),
                                                        )
                                                        .into()],
                                                        types::List::new(
                                                            output_element_type.clone(),
                                                            position.clone(),
                                                        ),
                                                        position.clone(),
                                                    )
                                                    .into(),
                                                ),
                                                Variable::new(CLOSURE_NAME, position.clone()),
                                                vec![Variable::new(LIST_NAME, position.clone())
                                                    .into()],
                                                position.clone(),
                                            )
                                            .into(),
                                        ),
                                    ],
                                    position.clone(),
                                ),
                                List::new(output_element_type.clone(), vec![], position.clone()),
                                position.clone(),
                            )
                            .into(),
                        )?,
                    ),
                    mir::ir::Call::new(
                        mir::types::Function::new(vec![list_type.clone().into()], list_type),
                        mir::ir::Variable::new(CLOSURE_NAME),
                        vec![compile(comprehension.iteratee())?],
                    ),
                ),
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
}
