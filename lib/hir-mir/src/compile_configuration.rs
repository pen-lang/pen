#[cfg(test)]
use super::{
    list_type_configuration::LIST_TYPE_CONFIGURATION,
    map_type_configuration::MAP_TYPE_CONFIGURATION,
    number_type_configuration::NUMBER_TYPE_CONFIGURATION,
    string_type_configuration::STRING_TYPE_CONFIGURATION,
};
use super::{
    list_type_configuration::ListTypeConfiguration, map_type_configuration::MapTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
use crate::number_type_configuration::NumberTypeConfiguration;
#[cfg(test)]
use std::sync::LazyLock;

#[derive(Clone, Debug)]
pub struct CompileConfiguration {
    pub list_type: ListTypeConfiguration,
    pub map_type: MapTypeConfiguration,
    pub number_type: NumberTypeConfiguration,
    pub string_type: StringTypeConfiguration,
    pub debug_function_name: String,
    pub race_function_name: String,
    pub spawn_function_name: String,
}

#[cfg(test)]
pub static COMPILE_CONFIGURATION: LazyLock<CompileConfiguration> =
    LazyLock::new(|| CompileConfiguration {
        list_type: LIST_TYPE_CONFIGURATION.clone(),
        map_type: MAP_TYPE_CONFIGURATION.clone(),
        number_type: NUMBER_TYPE_CONFIGURATION.clone(),
        string_type: STRING_TYPE_CONFIGURATION.clone(),
        debug_function_name: "debug".into(),
        race_function_name: "race".into(),
        spawn_function_name: "spawn".into(),
    });
