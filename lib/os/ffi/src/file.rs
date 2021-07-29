use crate::result::FfiResult;

use super::type_information;
use std::fs::File;
use std::sync::Arc;
use std::sync::LockResult;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

#[derive(Clone, Debug)]
pub struct FfiFile {
    file: Arc<RwLock<File>>,
}

type_information!(foo, crate::file::FfiFile);

impl FfiFile {
    pub fn new(file: File) -> Self {
        Self {
            file: Arc::new(RwLock::new(file)),
        }
    }

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, File>> {
        Ok(self.file.write()?)
    }
}
