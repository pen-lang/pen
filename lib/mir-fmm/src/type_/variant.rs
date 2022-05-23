use crate::type_;
use fnv::FnvHashMap;

pub fn compile(
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> fmm::types::Type {
    match type_ {
        mir::types::Type::Record(record_type) => {
            let type_ = type_::compile_record(record_type, types);

            if type_::is_record_boxed(record_type, types) == is_record_boxed(record_type, types) {
                type_
            } else {
                fmm::types::Pointer::new(type_).into()
            }
        }
        mir::types::Type::Variant => fmm::types::Pointer::new(type_::compile_variant()).into(),
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => type_::compile(type_, types),
    }
}

pub fn is_record_boxed(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> bool {
    !types[record.name()].fields().is_empty()
}
