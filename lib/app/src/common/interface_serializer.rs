use std::error::Error;

pub fn serialize(module: &interface::Module) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(bincode::serialize(&module)?)
}

pub fn deserialize(slice: &[u8]) -> Result<interface::Module, Box<dyn Error>> {
    Ok(bincode::deserialize(slice)?)
}
