use super::{
    error_type_configuration::ErrorTypeConfiguration,
    list_type_configuration::ListTypeConfiguration, map_type_configuration::MapTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
#[cfg(test)]
use super::{
    error_type_configuration::ERROR_TYPE_CONFIGURATION,
    list_type_configuration::LIST_TYPE_CONFIGURATION,
    map_type_configuration::MAP_TYPE_CONFIGURATION,
    string_type_configuration::STRING_TYPE_CONFIGURATION,
};
#[cfg(test)]
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct CompileConfiguration {
    pub list_type: ListTypeConfiguration,
    pub map_type: MapTypeConfiguration,
    pub string_type: StringTypeConfiguration,
    pub error_type: ErrorTypeConfiguration,
    pub spawn_function_name: String,
    pub debug_function_name: String,
}

#[cfg(test)]
pub static COMPILE_CONFIGURATION: Lazy<CompileConfiguration> = Lazy::new(|| CompileConfiguration {
    list_type: LIST_TYPE_CONFIGURATION.clone(),
    map_type: MAP_TYPE_CONFIGURATION.clone(),
    string_type: STRING_TYPE_CONFIGURATION.clone(),
    error_type: ERROR_TYPE_CONFIGURATION.clone(),
    spawn_function_name: "spawn".into(),
    debug_function_name: "debug".into(),
});
