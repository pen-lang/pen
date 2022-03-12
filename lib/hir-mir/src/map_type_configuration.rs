#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static MAP_TYPE_CONFIGURATION: Lazy<MapTypeConfiguration> =
    Lazy::new(|| MapTypeConfiguration {
        delete_function_name: "removeMap".into(),
        empty_function_name: "emptyMap".into(),
        empty_type_name: "notFound".into(),
        equal_function_name: "equalMaps".into(),
        get_function_name: "getMap".into(),
        map_type_name: "GenericMap".into(),
        merge_function_name: "mergeMaps".into(),
        set_function_name: "setMap".into(),
        hash: HASH_CONFIGURATION.clone(),
    });

#[cfg(test)]
pub static HASH_CONFIGURATION: Lazy<HashConfiguration> = Lazy::new(|| HashConfiguration {
    combine_function_name: "combineHashes".into(),
    number_hash_function_name: "hashNumber".into(),
    string_hash_function_name: "hashString".into(),
    list_hash_function_name: "hashList".into(),
    map_hash_function_name: "hashMap".into(),
});

#[derive(Clone, Debug, PartialEq)]
pub struct MapTypeConfiguration {
    pub delete_function_name: String,
    pub empty_function_name: String,
    pub empty_type_name: String,
    pub equal_function_name: String,
    pub get_function_name: String,
    pub map_type_name: String,
    pub merge_function_name: String,
    pub set_function_name: String,
    pub hash: HashConfiguration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HashConfiguration {
    pub combine_function_name: String,
    pub number_hash_function_name: String,
    pub string_hash_function_name: String,
    pub list_hash_function_name: String,
    pub map_hash_function_name: String,
}
