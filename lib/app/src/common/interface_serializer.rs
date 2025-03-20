use std::error::Error;

pub fn serialize(module: &interface::Module) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(bincode::serde::encode_to_vec(
        module,
        bincode::config::standard(),
    )?)
}

pub fn deserialize(slice: &[u8]) -> Result<interface::Module, Box<dyn Error>> {
    Ok(bincode::serde::decode_from_slice(slice, bincode::config::standard())?.0)
}
