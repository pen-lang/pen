use std::error::Error;

pub fn serialize(information: &test::Module) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}

pub fn deserialize(slice: &[u8]) -> Result<test::Module, Box<dyn Error>> {
    Ok(serde_json::from_slice(slice)?)
}
