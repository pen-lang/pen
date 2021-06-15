use crate::infra::{FilePath, FileSystem};
use std::error::Error;

pub struct BuildContext<M, MM, I> {
    pub file_system: Box<dyn FileSystem>,
}
