use crate::infra::FilePath;
use std::collections::HashMap;
use std::error::Error;

pub fn serialize(
    map: &HashMap<lang::ast::ModulePath, FilePath>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_string(
        &map.clone()
            .into_iter()
            .collect::<Vec<(lang::ast::ModulePath, FilePath)>>(),
    )?
    .into())
}

pub fn deserialize(
    slice: &[u8],
) -> Result<HashMap<lang::ast::ModulePath, FilePath>, Box<dyn Error>> {
    Ok(
        serde_json::from_slice::<Vec<(lang::ast::ModulePath, FilePath)>>(&slice)?
            .into_iter()
            .collect(),
    )
}
