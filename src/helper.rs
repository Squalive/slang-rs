use crate::FileSystem;

#[derive(Default)]
pub struct NeverFileSystem;

impl FileSystem for NeverFileSystem {
    fn load_file(&self, _path: &str) -> std::io::Result<Box<[u8]>> {
        Err(std::io::Error::other("never file system"))
    }
}
