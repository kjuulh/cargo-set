use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

pub trait FileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    #[inline(always)]
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }
}

#[allow(dead_code)]
pub struct MockFileSystem {
    files: HashMap<PathBuf, Vec<u8>>,
}

#[allow(dead_code)]
impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf, content: Vec<u8>) {
        self.files.insert(path, content);
    }
}

impl FileSystem for MockFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }
}
