use std::error::Error;

pub fn serialize(information: &hir_mir::TestModuleInformation) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}

pub fn deserialize(slice: &[u8]) -> Result<hir_mir::TestModuleInformation, Box<dyn Error>> {
    Ok(serde_json::from_slice(slice)?)
}
