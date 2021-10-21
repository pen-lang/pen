use crate::infra::FilePath;
use std::{collections::BTreeMap, error::Error};

type InterfaceFileMap = BTreeMap<ast::ModulePath, FilePath>;

pub fn serialize(
    interface_files: &InterfaceFileMap,
    prelude_interface_files: &[FilePath],
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(bincode::serialize(&(
        interface_files,
        prelude_interface_files,
    ))?)
}

pub fn deserialize(slice: &[u8]) -> Result<(InterfaceFileMap, Vec<FilePath>), Box<dyn Error>> {
    Ok(bincode::deserialize(slice)?)
}
