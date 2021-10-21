mod calls;
mod closures;
mod declarations;
mod definitions;
mod entry_functions;
mod error;
mod expressions;
mod foreign_declarations;
mod foreign_definitions;
mod records;
mod reference_count;
mod type_information;
mod types;
mod variants;

use declarations::compile_declaration;
use definitions::compile_definition;
pub use error::CompileError;
use foreign_declarations::compile_foreign_declaration;
use foreign_definitions::compile_foreign_definition;
use std::collections::BTreeMap;
use type_information::compile_type_information_global_variable;

pub fn compile(module: &mir::ir::Module) -> Result<fmm::ir::Module, CompileError> {
    mir::analysis::check_types(module)?;

    let module = mir::analysis::infer_environment(module);
    let module = mir::analysis::count_references(&module)?;

    mir::analysis::check_types(&module)?;

    let module_builder = fmm::build::ModuleBuilder::new();
    let types = module
        .type_definitions()
        .iter()
        .map(|definition| (definition.name().into(), definition.type_().clone()))
        .collect();

    for type_ in &mir::analysis::collect_variant_types(&module) {
        compile_type_information_global_variable(&module_builder, type_, &types)?;
    }

    for definition in module.type_definitions() {
        reference_count::compile_record_clone_function(&module_builder, definition, &types)?;
        reference_count::compile_record_drop_function(&module_builder, definition, &types)?;
    }

    for declaration in module.foreign_declarations() {
        compile_foreign_declaration(&module_builder, declaration, &types)?;
    }

    for declaration in module.declarations() {
        compile_declaration(&module_builder, declaration, &types);
    }

    let global_variables = compile_global_variables(&module, &types)?;

    for definition in module.definitions() {
        compile_definition(&module_builder, definition, &global_variables, &types)?;
    }

    let function_types = module
        .foreign_declarations()
        .iter()
        .map(|declaration| (declaration.name(), declaration.type_()))
        .chain(
            module
                .declarations()
                .iter()
                .map(|declaration| (declaration.name(), declaration.type_())),
        )
        .chain(
            module
                .definitions()
                .iter()
                .map(|definition| (definition.name(), definition.type_())),
        )
        .collect::<BTreeMap<_, _>>();

    for definition in module.foreign_definitions() {
        compile_foreign_definition(
            &module_builder,
            definition,
            function_types[definition.name()],
            &global_variables[definition.name()],
            &types,
        )?;
    }

    Ok(module_builder.as_module())
}

fn compile_global_variables(
    module: &mir::ir::Module,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<BTreeMap<String, fmm::build::TypedExpression>, CompileError> {
    module
        .foreign_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        declaration.type_(),
                        types,
                    )),
                ),
            )
        })
        .chain(module.declarations().iter().map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        declaration.type_(),
                        types,
                    )),
                ),
            )
        }))
        .chain(module.definitions().iter().map(|definition| {
            (
                definition.name().into(),
                fmm::build::bit_cast(
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        definition.type_(),
                        types,
                    )),
                    fmm::build::variable(
                        definition.name(),
                        fmm::types::Pointer::new(types::compile_sized_closure(definition, types)),
                    ),
                )
                .into(),
            )
        }))
        .map(|(name, expression)| Ok((name, reference_count::compile_tagged_pointer(&expression)?)))
        .collect::<Result<_, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_module(module: &mir::ir::Module) {
        let module = compile(module).unwrap();

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

    fn create_module_with_definitions(definitions: Vec<mir::ir::Definition>) -> mir::ir::Module {
        mir::ir::Module::new(vec![], vec![], vec![], vec![], definitions)
    }

    fn create_module_with_type_definitions(
        variant_definitions: Vec<mir::ir::TypeDefinition>,
        definitions: Vec<mir::ir::Definition>,
    ) -> mir::ir::Module {
        mir::ir::Module::new(variant_definitions, vec![], vec![], vec![], definitions)
    }

    #[test]
    fn compile_empty_module() {
        compile_module(&mir::ir::Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ));
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                    mir::ir::CallingConvention::Target,
                )],
                vec![],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![mir::ir::ForeignDeclaration::new(
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
                )],
                vec![],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_with_source_calling_convention() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                    mir::ir::CallingConvention::Source,
                )],
                vec![],
                vec![],
                vec![],
            ));
        }
    }

    mod foreign_definitions {
        use super::*;

        #[test]
        fn compile_for_foreign_declaration() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![mir::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                    mir::ir::CallingConvention::Target,
                )],
                vec![mir::ir::ForeignDefinition::new(
                    "f",
                    "h",
                    mir::ir::CallingConvention::Source,
                )],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_for_declaration() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![mir::ir::ForeignDefinition::new(
                    "f",
                    "g",
                    mir::ir::CallingConvention::Source,
                )],
                vec![mir::ir::Declaration::new(
                    "f",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                )],
                vec![],
            ));
        }

        #[test]
        fn compile_for_definition() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![mir::ir::ForeignDefinition::new(
                    "f",
                    "g",
                    mir::ir::CallingConvention::Source,
                )],
                vec![],
                vec![mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Variable::new("x"),
                    mir::types::Type::Number,
                )],
            ));
        }

        #[test]
        fn compile_for_definition_with_target_calling_convention() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![mir::ir::ForeignDefinition::new(
                    "f",
                    "g",
                    mir::ir::CallingConvention::Target,
                )],
                vec![],
                vec![mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Variable::new("x"),
                    mir::types::Type::Number,
                )],
            ));
        }
    }

    mod declarations {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![mir::ir::Declaration::new(
                    "f",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Type::Number,
                    ),
                )],
                vec![],
            ));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![mir::ir::Declaration::new(
                    "f",
                    mir::types::Function::new(
                        vec![mir::types::Type::Number],
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                    ),
                )],
                vec![],
            ));
        }
    }

    mod definitions {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Variable::new("x"),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::thunk(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Variable::new("x"),
                    mir::types::Type::Number,
                ),
                mir::ir::Definition::new(
                    "g",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::Call::new(
                        mir::types::Function::new(
                            vec![mir::types::Type::Number],
                            mir::types::Type::Number,
                        ),
                        mir::ir::Variable::new("f"),
                        vec![mir::ir::Variable::new("x").into()],
                    ),
                    mir::types::Type::Number,
                ),
            ]));
        }
    }

    mod expressions {
        use super::*;

        #[test]
        fn compile_let() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::new(
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
                            mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::new(
                            "g",
                            vec![mir::ir::Argument::new("y", mir::types::Type::Number)],
                            mir::ir::LetRecursive::new(
                                mir::ir::Definition::new(
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

        mod cases {
            use super::*;

            #[test]
            fn compile_with_float_64() {
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                mir::types::Type::Number,
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
            fn compile_with_unboxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                record_type.clone(),
                                "x",
                                mir::ir::Variable::new("x"),
                            )],
                            None,
                        ),
                        record_type,
                    )],
                ));
            }

            #[test]
            fn compile_with_boxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                record_type.clone(),
                                "x",
                                mir::ir::Variable::new("x"),
                            )],
                            None,
                        ),
                        record_type,
                    )],
                ));
            }

            #[test]
            fn compile_with_string() {
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                        mir::ir::Case::new(
                            mir::ir::Variable::new("x"),
                            vec![mir::ir::Alternative::new(
                                mir::types::Type::ByteString,
                                "y",
                                mir::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        mir::types::Type::ByteString,
                    ),
                ]));
            }
        }

        mod records {
            use super::*;

            #[test]
            fn compile_with_no_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Record::new(record_type.clone(), vec![]),
                        record_type,
                    )],
                ));
            }

            #[test]
            fn compile_with_1_field() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Record::new(record_type.clone(), vec![42.0.into()]),
                        record_type,
                    )],
                ));
            }

            #[test]
            fn compile_with_2_fields() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![
                            mir::types::Type::Number,
                            mir::types::Type::Boolean,
                        ]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Record::new(record_type.clone(), vec![42.0.into(), true.into()]),
                        record_type,
                    )],
                ));
            }

            #[test]
            fn compile_boxed() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Record::new(record_type.clone(), vec![42.0.into()]),
                        record_type,
                    )],
                ));
            }
        }

        mod record_fields {
            use super::*;

            #[test]
            fn compile_with_1_field_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", record_type.clone())],
                        mir::ir::RecordField::new(record_type, 0, mir::ir::Variable::new("x")),
                        mir::types::Type::Number,
                    )],
                ));
            }

            #[test]
            fn compile_with_2_field_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![
                            mir::types::Type::Boolean,
                            mir::types::Type::Number,
                        ]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", record_type.clone())],
                        mir::ir::RecordField::new(record_type, 1, mir::ir::Variable::new("x")),
                        mir::types::Type::Number,
                    )],
                ));
            }
        }

        mod variants {
            use super::*;

            #[test]
            fn compile_with_float_64() {
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
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

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", record_type.clone())],
                        mir::ir::Variant::new(
                            record_type.clone(),
                            mir::ir::Record::new(record_type, vec![]),
                        ),
                        mir::types::Type::Variant,
                    )],
                ));
            }

            #[test]
            fn compile_with_unboxed_record() {
                let record_type = mir::types::Record::new("foo");

                compile_module(&create_module_with_type_definitions(
                    vec![mir::ir::TypeDefinition::new(
                        "foo",
                        mir::types::RecordBody::new(vec![mir::types::Type::Number]),
                    )],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", record_type.clone())],
                        mir::ir::Variant::new(
                            record_type.clone(),
                            mir::ir::Record::new(record_type, vec![42.0.into()]),
                        ),
                        mir::types::Type::Variant,
                    )],
                ));
            }

            #[test]
            fn compile_with_string() {
                compile_module(&create_module_with_type_definitions(
                    vec![],
                    vec![mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variant::new(
                            mir::types::Type::ByteString,
                            mir::ir::ByteString::new("foo"),
                        ),
                        mir::types::Type::Variant,
                    )],
                ));
            }
        }

        mod calls {
            use super::*;

            #[test]
            fn compile_1_argument() {
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::Definition::new(
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
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![
                            mir::ir::Argument::new("x", mir::types::Type::Number),
                            mir::ir::Argument::new("y", mir::types::Type::Boolean),
                        ],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::Definition::new(
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
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![
                            mir::ir::Argument::new("x", mir::types::Type::Number),
                            mir::ir::Argument::new("y", mir::types::Type::Boolean),
                            mir::ir::Argument::new("z", mir::types::Type::ByteString),
                        ],
                        mir::ir::Variable::new("x"),
                        mir::types::Type::Number,
                    ),
                    mir::ir::Definition::new(
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
                compile_module(&create_module_with_definitions(vec![
                    mir::ir::Definition::new(
                        "f",
                        vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                        mir::ir::LetRecursive::new(
                            mir::ir::Definition::new(
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
                    mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::If::new(true, 42.0, 42.0),
                    mir::types::Type::Number,
                ),
            ]));
        }

        #[test]
        fn compile_try_operation() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::new(
                    "f",
                    vec![
                        mir::ir::Argument::new("x", mir::types::Type::ByteString),
                        mir::ir::Argument::new("y", mir::types::Type::ByteString),
                    ],
                    mir::ir::Expression::Number(42.0),
                    mir::types::Type::Number,
                ),
                mir::ir::Definition::new(
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
            compile_module(&create_module_with_type_definitions(
                vec![mir::ir::TypeDefinition::new(
                    "a",
                    mir::types::RecordBody::new(vec![]),
                )],
                vec![mir::ir::Definition::new(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Variant)],
                    mir::ir::Case::new(
                        mir::ir::Variable::new("x"),
                        vec![
                            mir::ir::Alternative::new(
                                mir::types::Type::ByteString,
                                "x",
                                mir::ir::Variable::new("x"),
                            ),
                            mir::ir::Alternative::new(
                                mir::types::Record::new("a"),
                                "x",
                                mir::ir::ByteString::new(vec![]),
                            ),
                        ],
                        None,
                    ),
                    mir::types::Type::ByteString,
                )],
            ));
        }
    }

    mod thunks {
        use super::*;

        #[test]
        fn compile_global_thunk() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::thunk(
                    "f",
                    vec![],
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                ),
            ]));
        }

        #[test]
        fn compile_local_thunk() {
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::thunk(
                    "f",
                    vec![],
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::thunk(
                            "g",
                            vec![],
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
            compile_module(&create_module_with_definitions(vec![
                mir::ir::Definition::thunk(
                    "f",
                    vec![mir::ir::Argument::new("x", mir::types::Type::Number)],
                    mir::ir::LetRecursive::new(
                        mir::ir::Definition::thunk(
                            "g",
                            vec![],
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
