use crate::types::{ProjectsError, ProjectsResult};
use std::{
    collections::HashMap,
    env,
    fs::{self, write, Metadata},
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
};

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> ProjectsResult<String>;
    fn write(&self, path: &Path, content: &str) -> ProjectsResult<()>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> ProjectsResult<()>;
    #[allow(dead_code)]
    fn metadata(&self, path: &Path) -> ProjectsResult<Metadata>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> ProjectsResult<String> {
        Ok(fs::read_to_string(path)?)
    }

  fn write(&self, path: &Path, content: &str) -> ProjectsResult<()> {
      path.parent()
          .map_or(Ok(()), |parent| self.create_dir_all(parent))
          .and_then(|_| Ok(write(path, content)?))
  }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> ProjectsResult<()> {
        Ok(fs::create_dir_all(path)?)
    }

    fn metadata(&self, path: &Path) -> ProjectsResult<Metadata> {
        Ok(fs::metadata(path)?)
    }
}

#[cfg(test)]
pub struct MockFileSystem {
    pub files: HashMap<PathBuf, String>,
    pub metadata_map: HashMap<PathBuf, MockMetadata>,
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct MockMetadata {
    pub len: u64,
    pub is_dir: bool,
    pub is_file: bool,
}

#[cfg(test)]
impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            metadata_map: HashMap::new(),
        }
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P, content: String) {
        let path_buf = path.as_ref().to_path_buf();
        let len = content.len() as u64;
        self.files.insert(path_buf.clone(), content);
        self.metadata_map.insert(path_buf, MockMetadata {
            len,
            is_file: true,
            is_dir: false,
        });
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, path: P) {
        let path_buf = path.as_ref().to_path_buf();
        self.metadata_map.insert(path_buf, MockMetadata {
            len: 0,
            is_file: false,
            is_dir: true,
        });
    }
}

#[cfg(test)]
impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &Path) -> ProjectsResult<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| ProjectsError::Io(IoError::new(
                ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )))
    }

    fn write(&self, _path: &Path, _content: &str) -> ProjectsResult<()> {
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn create_dir_all(&self, _path: &Path) -> ProjectsResult<()> {
        Ok(())
    }

    fn metadata(&self, path: &Path) -> ProjectsResult<Metadata> {
        let mock_metadata = self.metadata_map.get(path)
            .ok_or_else(|| ProjectsError::Io(IoError::new(
                ErrorKind::NotFound,
                format!("Metadata not found for path: {}", path.display()),
            )))?;

        let temp_file = env::temp_dir().join("mock_metadata_temp");
        if mock_metadata.is_dir {
            fs::create_dir_all(&temp_file)?;
            let metadata = fs::metadata(&temp_file)?;
            fs::remove_dir_all(&temp_file).ok();
            Ok(metadata)
        } else {
            let content = "x".repeat(mock_metadata.len as usize);
            fs::write(&temp_file, content)?;
            let metadata = fs::metadata(&temp_file)?;
            fs::remove_file(&temp_file).ok();
            Ok(metadata)
        }
    }
}