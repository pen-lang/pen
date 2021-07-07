use crate::infra::FilePath;
use std::{collections::HashMap, error::Error};

pub fn serialize(
    interface_files: &HashMap<lang::ast::ModulePath, FilePath>,
    prelude_interface_files: &[FilePath],
) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(bincode::serialize(&(interface_files, prelude_interface_files))?.into())
}

pub fn deserialize(
    slice: &[u8],
) -> Result<(HashMap<lang::ast::ModulePath, FilePath>, Vec<FilePath>), Box<dyn Error>> {
    Ok(bincode::deserialize(slice)?)
}
