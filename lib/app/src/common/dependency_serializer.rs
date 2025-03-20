use crate::infra::FilePath;
use std::{collections::BTreeMap, error::Error};

type InterfaceFileMap = BTreeMap<ast::ModulePath, FilePath>;

pub fn serialize(
    interface_files: &InterfaceFileMap,
    prelude_interface_files: &[FilePath],
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(bincode::serde::encode_to_vec(
        &(interface_files, prelude_interface_files),
        bincode::config::standard(),
    )?)
}

pub fn deserialize(slice: &[u8]) -> Result<(InterfaceFileMap, Vec<FilePath>), Box<dyn Error>> {
    Ok(bincode::serde::decode_from_slice(slice, bincode::config::standard())?.0)
}
