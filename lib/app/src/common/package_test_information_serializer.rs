use std::error::Error;

pub fn serialize(information: &test_info::Package) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}
