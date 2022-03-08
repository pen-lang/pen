#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static MAP_TYPE_CONFIGURATION: Lazy<MapTypeConfiguration> =
    Lazy::new(|| MapTypeConfiguration {
        empty_function_name: "emptyMap".into(),
        equal_function_name: "equalMaps".into(),
        map_type_name: "GenericMap".into(),
        merge_function_name: "mergeMaps".into(),
        delete_function_name: "removeMap".into(),
        set_function_name: "setMap".into(),
        hash: HASH_CONFIGURATION.clone(),
    });

#[cfg(test)]
pub static HASH_CONFIGURATION: Lazy<HashConfiguration> = Lazy::new(|| HashConfiguration {
    number_hash_function_name: "hashNumber".into(),
    string_hash_function_name: "hashString".into(),
    list_hash_function_name: "hashList".into(),
    map_hash_function_name: "hashMap".into(),
});

#[derive(Clone, Debug, PartialEq)]
pub struct MapTypeConfiguration {
    pub empty_function_name: String,
    pub equal_function_name: String,
    pub map_type_name: String,
    pub merge_function_name: String,
    pub delete_function_name: String,
    pub set_function_name: String,
    pub hash: HashConfiguration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HashConfiguration {
    pub number_hash_function_name: String,
    pub string_hash_function_name: String,
    pub list_hash_function_name: String,
    pub map_hash_function_name: String,
}
