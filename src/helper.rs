use crate::{FileSystem, ISlangCastable, ISlangFileSystem, ISlangUnknown, Uuid};
use alloc::vec::Vec;
use std::path::Path;

#[derive(Default)]
pub struct NeverFileSystem;

impl ISlangUnknown for NeverFileSystem {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        FileSystem::is_interface_compatible(uuid)
    }
}

impl ISlangCastable for NeverFileSystem {}

impl ISlangFileSystem for NeverFileSystem {
    fn load_file(&self, _path: &Path, _buf: &mut Vec<u8>) -> crate::Result<usize> {
        Err(crate::Error::Unknown)
    }
}
