use std::error::Error;

pub fn serialize(information: &test_info::Module) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}

pub fn deserialize(slice: &[u8]) -> Result<test_info::Module, Box<dyn Error>> {
    Ok(serde_json::from_slice(slice)?)
}
