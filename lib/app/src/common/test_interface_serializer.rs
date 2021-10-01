use std::{collections::BTreeMap, error::Error};

pub fn serialize(interface: &BTreeMap<String, String>) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&interface)?)
}

pub fn deserialize(slice: &[u8]) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    Ok(serde_json::from_slice(slice)?)
}
