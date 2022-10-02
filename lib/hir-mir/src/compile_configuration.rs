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
    number_type_configuration::NUMBER_TYPE_CONFIGURATION,
    string_type_configuration::STRING_TYPE_CONFIGURATION,
};
use crate::number_type_configuration::NumberTypeConfiguration;
#[cfg(test)]
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct CompileConfiguration {
    pub error_type: ErrorTypeConfiguration,
    pub list_type: ListTypeConfiguration,
    pub map_type: MapTypeConfiguration,
    pub number_type: NumberTypeConfiguration,
    pub string_type: StringTypeConfiguration,
    pub debug_function_name: String,
    pub race_function_name: String,
    pub spawn_function_name: String,
}

#[cfg(test)]
pub static COMPILE_CONFIGURATION: Lazy<CompileConfiguration> = Lazy::new(|| CompileConfiguration {
    error_type: ERROR_TYPE_CONFIGURATION.clone(),
    list_type: LIST_TYPE_CONFIGURATION.clone(),
    map_type: MAP_TYPE_CONFIGURATION.clone(),
    number_type: NUMBER_TYPE_CONFIGURATION.clone(),
    string_type: STRING_TYPE_CONFIGURATION.clone(),
    debug_function_name: "debug".into(),
    race_function_name: "race".into(),
    spawn_function_name: "spawn".into(),
});
