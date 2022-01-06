use super::{
    concurrency_configuration::ConcurrencyConfiguration,
    error_type_configuration::ErrorTypeConfiguration,
    list_type_configuration::ListTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};

#[derive(Clone, Debug)]
pub struct CompileConfiguration {
    pub list_type_configuration: ListTypeConfiguration,
    pub string_type_configuration: StringTypeConfiguration,
    pub error_type_configuration: ErrorTypeConfiguration,
    pub concurrency_configuration: ConcurrencyConfiguration,
}
