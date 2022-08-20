mod box_;
mod call;
mod closure;
mod configuration;
mod context;
mod entry_function;
mod error;
mod expression;
mod foreign_declaration;
mod foreign_definition;
mod foreign_value;
mod function_declaration;
mod function_definition;
mod pointer;
mod record;
mod reference_count;
mod type_;
mod type_information;
mod variant;
mod yield_;

pub use configuration::Configuration;
use context::Context;
pub use error::CompileError;
use fnv::FnvHashMap;

pub fn compile(
    module: &mir::ir::Module,
    configuration: &Configuration,
) -> Result<fmm::ir::Module, CompileError> {
    mir::analysis::type_check::check(module)?;

    let module = mir::analysis::environment_inference::transform(module);
    let module = mir::analysis::lambda_lifting::transform(&module);
    let module = mir::analysis::reference_count::transform(&module)?;

    mir::analysis::type_check::check(&module)?;

    let context = Context::new(&module, configuration.clone());

    for type_ in &mir::analysis::variant_type_collection::collect(&module) {
        type_information::compile_global_variable(&context, type_)?;
    }

    for definition in module.type_definitions() {
        reference_count::record::compile_clone_function(&context, definition)?;
        reference_count::record::compile_drop_function(&context, definition)?;
        reference_count::record::compile_synchronize_function(&context, definition)?;
    }

    for declaration in module.foreign_declarations() {
        foreign_declaration::compile(&context, declaration)?;
    }

    for declaration in module.function_declarations() {
        function_declaration::compile(&context, declaration);
    }

    let global_variables = compile_global_variables(&module, context.types())?;

    for definition in module.function_definitions() {
        function_definition::compile(&context, definition, &global_variables)?;
    }

    let function_types = module
        .foreign_declarations()
        .iter()
        .map(|declaration| (declaration.name(), declaration.type_()))
        .chain(
            module
                .function_declarations()
                .iter()
                .map(|declaration| (declaration.name(), declaration.type_())),
        )
        .chain(module.function_definitions().iter().map(|definition| {
            (
                definition.definition().name(),
                definition.definition().type_(),
            )
        }))
        .collect::<FnvHashMap<_, _>>();

    for definition in module.foreign_definitions() {
        foreign_definition::compile(
            &context,
            definition,
            function_types[definition.name()],
            &global_variables[definition.name()],
        )?;
    }

    yield_::compile_function_declaration(&context);

    Ok(context.module_builder().as_module())
}

fn compile_global_variables(
    module: &mir::ir::Module,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<FnvHashMap<String, fmm::build::TypedExpression>, CompileError> {
    module
        .foreign_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(reference_count::block::compile_type(
                        type_::compile_unsized_closure(declaration.type_(), types),
                    )),
                ),
            )
        })
        .chain(module.function_declarations().iter().map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(reference_count::block::compile_type(
                        type_::compile_unsized_closure(declaration.type_(), types),
                    )),
                ),
            )
        }))
        .chain(module.function_definitions().iter().map(|definition| {
            let definition = definition.definition();

            (
                definition.name().into(),
                fmm::build::bit_cast(
                    fmm::types::Pointer::new(reference_count::block::compile_type(
                        type_::compile_unsized_closure(definition.type_(), types),
                    )),
                    fmm::build::variable(
                        definition.name(),
                        fmm::types::Pointer::new(reference_count::block::compile_type(
                            type_::compile_sized_closure(definition, types),
                        )),
                    ),
                )
                .into(),
            )
        }))
        .map(|(name, expression)| Ok((name, fmm::build::record_address(expression, 1)?.into())))
        .collect::<Result<_, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::CONFIGURATION;
    use mir::test::{FunctionDefinitionFake, ModuleFake};
    use once_cell::sync::Lazy;

    static FOREIGN_UNBOXED_RECORD_DEFINITION: Lazy<mir::ir::TypeDefinition> = Lazy::new(|| {
        mir::ir::TypeDefinition::new(
            "a",
            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
        )
    });

    static VARIANT_UNBOXED_RECORD_DEFINITION: Lazy<mir::ir::TypeDefinition> = Lazy::new(|| {
        mir::ir::TypeDefinition::new(
            "a",
            mir::types::RecordBody::new(vec![mir::types::Type::Number, mir::types::Type::Number]),
        )
    });

    fn compile_module(module: &mir::ir::Module) {
        let module = compile(module, &CONFIGURATION).unwrap();

        compile_final_module(&module);
        compile_final_module(
            &fmm::analysis::transform_to_cps(&module, fmm::types::Record::new(vec![])).unwrap(),
        );
    }

    fn compile_final_module(module: &fmm::ir::Module) {
        fmm::analysis::check_types(module).unwrap();

        fmm_llvm::compile_to_object(
            module,
            &fmm_llvm::InstructionConfiguration {
                allocate_function_name: "allocate_heap".into(),
                reallocate_function_name: "reallocate_heap".into(),
                free_function_name: "free_heap".into(),
                unreachable_function_name: None,
            },
            None,
        )
        .unwrap();
    }

    #[test]
    fn compile_empty_module() {
        compile_module(&mir::ir::Module::empty());
    }

    mod foreign_declaration {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&mir::ir::Module::empty().set_foreign_declarations(vec![
                mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                    mir::ir::CallingConvention::Target,
                ),
            ]));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&mir::ir::Module::empty().set_foreign_declarations(vec![
                mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                    ),
                    mir::ir::CallingConvention::Target,
                ),
            ]));
        }

        #[test]
        fn compile_with_source_calling_convention() {
            compile_module(&mir::ir::Module::empty().set_foreign_declarations(vec![
                mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                    mir::ir::CallingConvention::Source,
                ),
            ]));
        }

        #[test]
        fn compile_with_variant_argument() {
            compile_module(&mir::ir::Module::empty().set_foreign_declarations(vec![
                mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Variant],
                        mir::types::Type::None,
                    ),
                    mir::ir::CallingConvention::Target,
                ),
            ]));
        }

        #[test]
        fn compile_with_variant_result() {
            compile_module(&mir::ir::Module::empty().set_foreign_declarations(vec![
                mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(vec![], mir::types::Type::Variant),
                    mir::ir::CallingConvention::Target,
                ),
            ]));
        }

        #[test]
        fn compile_with_unboxed_record_argument() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_type_definitions(vec![FOREIGN_UNBOXED_RECORD_DEFINITION.clone()])
                    .set_foreign_declarations(vec![mir::ir::ForeignDeclaration::new(
                        "f",
                        "g",
                        mir::types::Function::new(
                            vec![mir::types::Record::new("a").into()],
                            mir::types::Type::None,
                        ),
                        mir::ir::CallingConvention::Target,
                    )]),
            );
        }

        #[test]
        fn compile_with_unboxed_record_result() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_type_definitions(vec![FOREIGN_UNBOXED_RECORD_DEFINITION.clone()])
                    .set_foreign_declarations(vec![mir::ir::ForeignDeclaration::new(
                        "f",
                        "g",
                        mir::types::Function::new(vec![], mir::types::Record::new("a")),
                        mir::ir::CallingConvention::Target,
                    )]),
            );
        }
    }

    mod foreign_definition {
        use super::*;

        #[test]
        fn compile_for_foreign_declaration() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_declarations(vec![mir::ir::ForeignDeclaration::new(
                        "f",
                        "g",
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "h",
                        mir::ir::CallingConvention::Source,
                    )]),
            );
        }

        #[test]
        fn compile_for_declaration() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Source,
                    )])
                    .set_function_declarations(vec![mir::ir::FunctionDeclaration::new(
                        "f",
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                    )]),
            );
        }

        #[test]
        fn compile_for_definition() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Source,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    )]),
            );
        }

        #[test]
        fn compile_for_definition_with_target_calling_convention() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    )]),
            );
        }

        #[test]
        fn compile_with_variant_argument() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Expression::None,
                        mir::types::Type::None,
                    )]),
            );
        }

        #[test]
        fn compile_with_variant_result() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![],
                        mir::ir::Variant::new(mir::types::Type::None, mir::ir::Expression::None),
                        mir::types::Type::Variant,
                    )]),
            );
        }

        #[test]
        fn compile_with_unboxed_record_argument() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_type_definitions(vec![FOREIGN_UNBOXED_RECORD_DEFINITION.clone()])
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Record::new("a"))],
                        mir::ir::Expression::None,
                        mir::types::Type::None,
                    )]),
            );
        }

        #[test]
        fn compile_with_unboxed_record_result() {
            let record_type = mir::types::Record::new("a");

            compile_module(
                &mir::ir::Module::empty()
                    .set_type_definitions(vec![FOREIGN_UNBOXED_RECORD_DEFINITION.clone()])
                    .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                        "f",
                        "g",
                        mir::ir::CallingConvention::Target,
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![],
                        mir::ir::Record::new(record_type, vec![mir::ir::Expression::Number(42.0)]),
                        mir::types::Record::new("a"),
                    )]),
            );
        }
    }

    mod declaration {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&mir::ir::Module::empty().set_function_declarations(vec![
                mir::ir::FunctionDeclaration::new(
                    "f",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                ),
            ]));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&mir::ir::Module::empty().set_function_declarations(vec![
                mir::ir::FunctionDeclaration::new(
                    "f",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                    ),
                ),
            ]));
        }
    }

    mod definition {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Variable::new("x"),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![
                        mir::ir::Argument::new("x", mir::types::Type::Number),
                        mir::ir::Argument::new("y", mir::types::Type::Number),
                    ],
                    mir::ir::ArithmeticOperation::new(
                        mir::ir::ArithmeticOperator::Add,
                        mir::ir::Variable::new("x"),
                        mir::ir::Variable::new("y"),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_thunk() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake_thunk(
                    "f",
                    mir::ir::Expression::Number(42.0),
                    mir::types::Type::Number,
                ),
                mir::ir::FunctionDefinition::fake(
                    "g",
                    vec![],
                    mir::ir::Call::new(
                        mir::types::Function::new(vec![], mir::types::Type::Number),
                        mir::ir::Variable::new("f"),
                        vec![],
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }
    }

    mod expression {
        use super::*;

        #[test]
        fn compile_let() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Let::new(
                        "y",
                        mir::types::Type::Number,
                        mir::ir::Variable::new("x"),
                        mir::ir::Variable::new("y"),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_let_recursive() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::fake(
                            "g",
                            vec![mir::ir::Argument::new("y", mir::types::Type::Number)],
                            mir::ir::ArithmeticOperation::new(
                                mir::ir::ArithmeticOperator::Add,
                                mir::ir::Variable::new("x"),
                                mir::ir::Variable::new("y"),
                            ),
                            mir::types::Type::Number,
                        ),
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![mir::types::Type::Number],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Variable::new("g"),
                            vec![42.0.into()],
                        ),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_nested_let_recursive() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::fake(
                            "g",
                            vec![mir::ir::Argument::new("y", mir::types::Type::Number)],
                            mir::ir::ArithmeticOperation::new(
                                mir::ir::ArithmeticOperator::Add,
                                mir::ir::Variable::new("x"),
                                mir::ir::Variable::new("y"),
                            ),
                            mir::types::Type::Number,
                        ),
                        mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::fake(
                                "h",
                                vec![mir::ir::Argument::new("z", mir::types::Type::Number)],
                                mir::ir::Call::new(
                                    mir::types::Function::new(
                                        vec![mir::types::Type::Number],
                                        mir::types::Type::Number,
                                    ),
                                    mir::ir::Variable::new("g"),
                                    vec![mir::ir::Variable::new("z").into()],
                                ),
                                mir::types::Type::Number,
                            ),
                            mir::ir::Call::new(
                                mir::types::Function::new(
                                    vec![mir::types::Type::Number],
                                    mir::types::Type::Number,
                                ),
                                mir::ir::Variable::new("h"),
                                vec![42.0.into()],
                            ),
                        ),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_let_recursive_with_curried_function() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::fake(
                            "g",
                            vec![mir::ir::Argument::new("y", mir::types::Type::Number)],
                            mir::ir::LetRecursive::new(
                                mir::ir::FunctionDefinition::fake(
                                    "h",
                                    vec![mir::ir::Argument::new("z", mir::types::Type::Number)],
                                    mir::ir::ArithmeticOperation::new(
                                        mir::ir::ArithmeticOperator::Add,
                                        mir::ir::ArithmeticOperation::new(
                                            mir::ir::ArithmeticOperator::Add,
                                            mir::ir::Variable::new("x"),
                                            mir::ir::Variable::new("y"),
                                        ),
                                        mir::ir::Variable::new("z"),
                                    ),
                                    mir::types::Type::Number,
                                ),
                                mir::ir::Variable::new("h"),
                            ),
                            mir::types::Function::new(
                                vec![mir::types::Type::Number],
                                mir::types::Type::Number,
                            ),
                        ),
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![mir::types::Type::Number],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Call::new(
                                mir::types::Function::new(
                                    vec![mir::types::Type::Number],
                                    mir::types::Function::new(
                                        vec![mir::types::Type::Number],
                                        mir::types::Type::Number,
                                    ),
                                ),
                                mir::ir::Variable::new("g"),
                                vec![42.0.into()],
                            ),
                            vec![42.0.into()],
                        ),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        mod case {
            use super::*;

            #[test]
            fn compile_number() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                vec![mir::types::Type::Number],
                                "y",
                                mir::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        mir::types::Type::Number,
                    ),
                ]));
            }

            #[test]
            fn compile_unboxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                            mir::ir::Case::new(
                                mir::ir::Variable::new("x"),
                                vec![mir::ir::Alternative::new(
                                    vec![record_type.clone().into()],
                                    "x",
                                    mir::ir::Variable::new("x"),
                                )],
                                None,
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_boxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                            mir::ir::Case::new(
                                mir::ir::Variable::new("x"),
                                vec![mir::ir::Alternative::new(
                                    vec![record_type.clone().into()],
                                    "x",
                                    mir::ir::Variable::new("x"),
                                )],
                                None,
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_string() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                vec![mir::types::Type::ByteString],
                                "y",
                                mir::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        mir::types::Type::ByteString,
                    ),
                ]));
            }

            #[test]
            fn compile_unboxed_large_record() {
                let record_type = mir::types::Record::new("a");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![VARIANT_UNBOXED_RECORD_DEFINITION.clone()])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                            mir::ir::Case::new(
                                mir::ir::Variable::new("x"),
                                vec![mir::ir::Alternative::new(
                                    vec![record_type.clone().into()],
                                    "y",
                                    mir::ir::Variable::new("y"),
                                )],
                                None,
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_multiple_types() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                vec![mir::types::Type::Number, mir::types::Type::None],
                                "y",
                                mir::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        mir::types::Type::Variant,
                    ),
                ]));
            }
        }

        mod record {
            use super::*;

            #[test]
            fn compile_with_no_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                            mir::ir::Record::new(record_type.clone(), vec![]),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_with_1_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                            mir::ir::Record::new(record_type.clone(), vec![42.0.into()]),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_with_2_fields() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::Number,
                                mir::types::Type::Boolean,
                            ]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                            mir::ir::Record::new(
                                record_type.clone(),
                                vec![42.0.into(), true.into()],
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_boxed() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                            mir::ir::Record::new(record_type.clone(), vec![42.0.into()]),
                            record_type,
                        )]),
                );
            }
        }

        mod record_field {
            use super::*;

            #[test]
            fn compile_with_1_field_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", record_type.clone())],
                            mir::ir::RecordField::new(record_type, 0, mir::ir::Variable::new("x")),
                            mir::types::Type::Number,
                        )]),
                );
            }

            #[test]
            fn compile_with_2_field_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::Boolean,
                                mir::types::Type::Number,
                            ]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", record_type.clone())],
                            mir::ir::RecordField::new(record_type, 1, mir::ir::Variable::new("x")),
                            mir::types::Type::Number,
                        )]),
                );
            }
        }

        mod record_update {
            use super::*;

            #[test]
            fn compile_with_empty_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![],
                            mir::ir::RecordUpdate::new(
                                record_type.clone(),
                                mir::ir::Record::new(record_type.clone(), vec![]),
                                vec![],
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_record_with_1_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![],
                            mir::ir::RecordUpdate::new(
                                record_type.clone(),
                                mir::ir::Record::new(record_type.clone(), vec![42.0.into()]),
                                vec![mir::ir::RecordUpdateField::new(0, 0.0)],
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_with_1_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::Number,
                                mir::types::Type::Boolean,
                            ]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![],
                            mir::ir::RecordUpdate::new(
                                record_type.clone(),
                                mir::ir::Record::new(
                                    record_type.clone(),
                                    vec![42.0.into(), true.into()],
                                ),
                                vec![mir::ir::RecordUpdateField::new(1, false)],
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_with_2_fields() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::Number,
                                mir::types::Type::Boolean,
                                mir::types::Type::None,
                            ]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![],
                            mir::ir::RecordUpdate::new(
                                record_type.clone(),
                                mir::ir::Record::new(
                                    record_type.clone(),
                                    vec![42.0.into(), true.into(), mir::ir::Expression::None],
                                ),
                                vec![
                                    mir::ir::RecordUpdateField::new(1, false),
                                    mir::ir::RecordUpdateField::new(2, mir::ir::Expression::None),
                                ],
                            ),
                            record_type,
                        )]),
                );
            }

            #[test]
            fn compile_with_swapped_2_fields() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::Number,
                                mir::types::Type::Boolean,
                                mir::types::Type::None,
                            ]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![],
                            mir::ir::RecordUpdate::new(
                                record_type.clone(),
                                mir::ir::Record::new(
                                    record_type.clone(),
                                    vec![42.0.into(), true.into(), mir::ir::Expression::None],
                                ),
                                vec![
                                    mir::ir::RecordUpdateField::new(2, mir::ir::Expression::None),
                                    mir::ir::RecordUpdateField::new(1, false),
                                ],
                            ),
                            record_type,
                        )]),
                );
            }
        }

        mod variant {
            use super::*;

            #[test]
            fn compile_with_float_64() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variant::new(mir::types::Type::Number, 42.0),
                        mir::types::Type::Variant,
                    ),
                ]));
            }

            #[test]
            fn compile_with_empty_unboxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", record_type.clone())],
                            mir::ir::Variant::new(
                                record_type.clone(),
                                mir::ir::Record::new(record_type, vec![]),
                            ),
                            mir::types::Type::Variant,
                        )]),
                );
            }

            #[test]
            fn compile_with_unboxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                            "foo",
                            mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                        )])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", record_type.clone())],
                            mir::ir::Variant::new(
                                record_type.clone(),
                                mir::ir::Record::new(record_type, vec![42.0.into()]),
                            ),
                            mir::types::Type::Variant,
                        )]),
                );
            }

            #[test]
            fn compile_with_string() {
                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                            mir::ir::Variant::new(
                                mir::types::Type::ByteString,
                                mir::ir::ByteString::new("foo"),
                            ),
                            mir::types::Type::Variant,
                        )]),
                );
            }

            #[test]
            fn compile_unboxed_large_record() {
                let record_type = mir::types::Record::new("a");

                compile_module(
                    &mir::ir::Module::empty()
                        .set_type_definitions(vec![VARIANT_UNBOXED_RECORD_DEFINITION.clone()])
                        .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                            "f",
                            vec![mir::ir::Argument::new("x", record_type.clone())],
                            mir::ir::Variant::new(record_type, mir::ir::Variable::new("x")),
                            mir::types::Type::Variant,
                        )]),
                );
            }
        }

        mod calls {
            use super::*;

            #[test]
            fn compile_1_argument() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::FunctionDefinition::fake(
                        "g",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![mir::types::Type::Number],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Variable::new("f"),
                            vec![42.0.into()],
                        ),
                        mir::types::Type::Number,
                    ),
                ]));
            }

            #[test]
            fn compile_2_arguments() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![
                            mir::ir::Argument::new("x", mir::types::Type::Number),
                            mir::ir::Argument::new("y", mir::types::Type::Boolean),
                        ],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::FunctionDefinition::fake(
                        "g",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![mir::types::Type::Number, mir::types::Type::Boolean],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Variable::new("f"),
                            vec![42.0.into(), true.into()],
                        ),
                        mir::types::Type::Number,
                    ),
                ]));
            }

            #[test]
            fn compile_3_arguments() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![
                            mir::ir::Argument::new("x", mir::types::Type::Number),
                            mir::ir::Argument::new("y", mir::types::Type::Boolean),
                            mir::ir::Argument::new("z", mir::types::Type::ByteString),
                        ],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::FunctionDefinition::fake(
                        "g",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![
                                    mir::types::Type::Number,
                                    mir::types::Type::Boolean,
                                    mir::types::Type::ByteString,
                                ],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Variable::new("f"),
                            vec![
                                42.0.into(),
                                true.into(),
                                mir::ir::ByteString::new("foo").into(),
                            ],
                        ),
                        mir::types::Type::Number,
                    ),
                ]));
            }

            #[test]
            fn compile_with_curried_function() {
                compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                    mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::fake(
                                "g",
                                vec![mir::ir::Argument::new("y", mir::types::Type::Number)],
                                mir::ir::ArithmeticOperation::new(
                                    mir::ir::ArithmeticOperator::Add,
                                    mir::ir::Variable::new("x"),
                                    mir::ir::Variable::new("y"),
                                ),
                                mir::types::Type::Number,
                            ),
                            mir::ir::Variable::new("g"),
                        ),
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                    ),
                    mir::ir::FunctionDefinition::fake(
                        "g",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![mir::types::Type::Number],
                                mir::types::Type::Number,
                            ),
                            mir::ir::Call::new(
                                mir::types::Function::new(
                                    vec![mir::types::Type::Number],
                                    mir::types::Function::new(
                                        vec![mir::types::Type::Number],
                                        mir::types::Type::Number,
                                    ),
                                ),
                                mir::ir::Variable::new("f"),
                                vec![111.0.into()],
                            ),
                            vec![222.0.into()],
                        ),
                        mir::types::Type::Number,
                    ),
                ]));
            }
        }

        #[test]
        fn compile_if() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::If::new(true, 42.0, 42.0),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_try_operation() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                    mir::ir::TryOperation::new(
                        mir::ir::Variable::new("x"),
                        "y",
                        mir::types::Type::Number,
                        mir::ir::Variant::new(
                            mir::types::Type::Number,
                            mir::ir::Variable::new("y"),
                        ),
                    ),
                    mir::types::Type::Variant,
                ),
            ]));
        }
    }

    mod reference_count {
        use super::*;

        #[test]
        fn clone_and_drop_strings() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![
                        mir::ir::Argument::new("x", mir::types::Type::ByteString),
                        mir::ir::Argument::new("y", mir::types::Type::ByteString),
                    ],
                    mir::ir::Expression::Number(42.0),
                    mir::types::Type::Number,
                ),
                mir::ir::FunctionDefinition::fake(
                    "g",
                    vec![mir::ir::Argument::new("x", mir::types::Type::ByteString)],
                    mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![mir::types::Type::ByteString, mir::types::Type::ByteString],
                            mir::types::Type::Number,
                        ),
                        mir::ir::Variable::new("f"),
                        vec![
                            mir::ir::Variable::new("x").into(),
                            mir::ir::Variable::new("x").into(),
                        ],
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn drop_variable_captured_in_other_alternative_in_case() {
            compile_module(
                &mir::ir::Module::empty()
                    .set_type_definitions(vec![mir::ir::TypeDefinition::new(
                        "a",
                        mir::types::RecordBody::new(vec![]),
                    )])
                    .set_function_definitions(vec![mir::ir::FunctionDefinition::fake(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![
                                mir::ir::Alternative::new(
                                    vec![mir::types::Type::ByteString],
                                    "x",
                                    mir::ir::Variable::new("x"),
                                ),
                                mir::ir::Alternative::new(
                                    vec![mir::types::Record::new("a").into()],
                                    "x",
                                    mir::ir::ByteString::new(vec![]),
                                ),
                            ],
                            None,
                        ),
                        mir::types::Type::ByteString,
                    )]),
            );
        }
    }

    mod thunks {
        use super::*;

        #[test]
        fn compile_global_thunk() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake_thunk(
                    "f",
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                ),
            ]));
        }

        #[test]
        fn compile_local_thunk() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake_thunk(
                    "f",
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::fake_thunk(
                            "g",
                            mir::ir::Expression::None,
                            mir::types::Type::None,
                        ),
                        mir::ir::Call::new(
                            mir::types::Function::new(vec![], mir::types::Type::None),
                            mir::ir::Variable::new("g"),
                            vec![],
                        ),
                    ),
                    mir::types::Type::None,
                ),
            ]));
        }

        #[test]
        fn compile_local_thunk_with_environment() {
            compile_module(&mir::ir::Module::empty().set_function_definitions(vec![
                mir::ir::FunctionDefinition::fake(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::fake_thunk(
                            "g",
                            mir::ir::Variable::new("x"),
                            mir::types::Type::Number,
                        ),
                        mir::ir::Call::new(
                            mir::types::Function::new(vec![], mir::types::Type::Number),
                            mir::ir::Variable::new("g"),
                            vec![],
                        ),
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }
    }
}
