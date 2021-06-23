use std::error::Error;

pub fn serialize(module: &lang::interface::Module) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_string(&module)?.into())
}

pub fn deserialize(slice: &[u8]) -> Result<lang::interface::Module, Box<dyn Error>> {
    Ok(serde_json::from_slice(slice)?)
}
