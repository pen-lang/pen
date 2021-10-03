use std::{collections::BTreeMap, error::Error};

pub fn serialize(
    information: &BTreeMap<String, hir_mir::TestModuleInformation>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec(&information)?)
}
