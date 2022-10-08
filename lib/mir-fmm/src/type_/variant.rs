use crate::{context::Context, type_, CompileError};

pub fn compile_payload(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::types::Type, CompileError> {
    let fmm_type = type_::compile(context, type_);

    Ok(if is_payload_boxed(context, type_)? {
        fmm::types::Pointer::new(fmm_type).into()
    } else {
        fmm_type
    })
}

pub fn is_payload_boxed(context: &Context, type_: &mir::types::Type) -> Result<bool, CompileError> {
    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(context, record_type)
                && !is_record_boxed(context, record_type)
            {
                return Err(CompileError::UnboxedRecord);
            }

            type_::is_record_boxed(context, record_type) != is_record_boxed(context, record_type)
        }
        mir::types::Type::Variant => return Err(CompileError::NestedVariant),
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => false,
    })
}

// Box large records to stuff them into one word.
fn is_record_boxed(context: &Context, record: &mir::types::Record) -> bool {
    let body_type = &context.types()[record.name()];

    body_type.fields().len() > 1
        || body_type.fields().iter().any(|type_| match type_ {
            mir::types::Type::Record(record) => is_record_boxed(context, record),
            // Variants always take two words.
            mir::types::Type::Variant => true,
            _ => false,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::CONFIGURATION;
    use mir::test::ModuleFake;

    mod record_box {
        use super::*;

        #[test]
        fn check_empty_record() {
            assert!(!is_record_boxed(
                &Context::new(
                    &mir::ir::Module::empty().set_type_definitions(vec![
                        mir::ir::TypeDefinition::new("r", mir::types::RecordBody::new(vec![]))
                    ]),
                    CONFIGURATION.clone()
                ),
                &mir::types::Record::new("r")
            ));
        }

        #[test]
        fn check_record_with_field() {
            assert!(!is_record_boxed(
                &Context::new(
                    &mir::ir::Module::empty().set_type_definitions(vec![
                        mir::ir::TypeDefinition::new(
                            "r",
                            mir::types::RecordBody::new(vec![mir::types::Type::None])
                        )
                    ]),
                    CONFIGURATION.clone()
                ),
                &mir::types::Record::new("r")
            ));
        }

        #[test]
        fn check_record_with_variant_field() {
            assert!(is_record_boxed(
                &Context::new(
                    &mir::ir::Module::empty().set_type_definitions(vec![
                        mir::ir::TypeDefinition::new(
                            "r",
                            mir::types::RecordBody::new(vec![mir::types::Type::Variant])
                        )
                    ]),
                    CONFIGURATION.clone()
                ),
                &mir::types::Record::new("r")
            ));
        }

        #[test]
        fn check_record_with_field_of_record_with_variant_field() {
            assert!(is_record_boxed(
                &Context::new(
                    &mir::ir::Module::empty().set_type_definitions(vec![
                        mir::ir::TypeDefinition::new(
                            "r",
                            mir::types::RecordBody::new(vec![mir::types::Type::Variant])
                        ),
                        mir::ir::TypeDefinition::new(
                            "s",
                            mir::types::RecordBody::new(vec![mir::types::Record::new("r").into()])
                        )
                    ]),
                    CONFIGURATION.clone()
                ),
                &mir::types::Record::new("s")
            ));
        }

        #[test]
        fn check_record_with_two_fields() {
            assert!(is_record_boxed(
                &Context::new(
                    &mir::ir::Module::empty().set_type_definitions(vec![
                        mir::ir::TypeDefinition::new(
                            "r",
                            mir::types::RecordBody::new(vec![
                                mir::types::Type::None,
                                mir::types::Type::None
                            ])
                        )
                    ]),
                    CONFIGURATION.clone()
                ),
                &mir::types::Record::new("r")
            ));
        }
    }
}
