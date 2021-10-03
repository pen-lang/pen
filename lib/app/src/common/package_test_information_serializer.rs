use std::{collections::BTreeMap, error::Error};

pub fn serialize(
    information: &BTreeMap<String, test_info::Module>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}
