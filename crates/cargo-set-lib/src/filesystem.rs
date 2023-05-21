use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait FileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, contents: Vec<u8>) -> io::Result<()>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    #[inline(always)]
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    #[inline(always)]
    fn write(&self, path: &Path, contents: Vec<u8>) -> io::Result<()> {
        std::fs::write(path, contents)
    }
}

#[allow(dead_code)]
pub struct MockFileSystem {
    files: Mutex<HashMap<PathBuf, Vec<u8>>>,
}

#[allow(dead_code)]
impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, content: Vec<u8>) {
        let mut files = self.files.lock().unwrap();
        files.insert(path, content);
    }
}

impl FileSystem for MockFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }

    fn write(&self, path: &Path, contents: Vec<u8>) -> io::Result<()> {
        let mut files = self.files.lock().unwrap();
        let file = files
            .get_mut(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))?
            .as_mut();

        *file = contents;

        Ok(())
    }
}
