use std::path::Path;
use crate::types::ProjectsResult;
use std::fs::{write, Metadata};

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> ProjectsResult<String>;
    fn write(&self, path: &Path, content: &str) -> ProjectsResult<()>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> ProjectsResult<()>;
    #[allow(dead_code)]
    fn metadata(&self, path: &Path) -> ProjectsResult<std::fs::Metadata>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> ProjectsResult<String> {
        Ok(std::fs::read_to_string(path)?)
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
        Ok(std::fs::create_dir_all(path)?)
    }

    fn metadata(&self, path: &Path) -> ProjectsResult<Metadata> {
        Ok(std::fs::metadata(path)?)
    }
}

#[cfg(test)]
pub struct MockFileSystem {
    pub files: std::collections::HashMap<std::path::PathBuf, String>,
    pub metadata_map: std::collections::HashMap<std::path::PathBuf, MockMetadata>,
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
            files: std::collections::HashMap::new(),
            metadata_map: std::collections::HashMap::new(),
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
            .ok_or_else(|| crate::types::ProjectsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
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

    fn metadata(&self, path: &Path) -> ProjectsResult<std::fs::Metadata> {
        // For testing, we need to create a mock metadata object
        // Since std::fs::Metadata is not constructible directly, we use a workaround:
        // Create a temporary file to get real metadata, then we'll have to work around this limitation
        
        let mock_metadata = self.metadata_map.get(path)
            .ok_or_else(|| crate::types::ProjectsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Metadata not found for path: {}", path.display()),
            )))?;

        // Since we can't construct std::fs::Metadata directly, we'll use a temporary file approach
        // This is a workaround for testing - in a real implementation you might want to create
        // a trait for metadata operations too
        let temp_file = std::env::temp_dir().join("mock_metadata_temp");
        if mock_metadata.is_dir {
            std::fs::create_dir_all(&temp_file)?;
            let metadata = std::fs::metadata(&temp_file)?;
            std::fs::remove_dir_all(&temp_file).ok(); // Clean up
            Ok(metadata)
        } else {
            // Create a temporary file with the right size
            let content = "x".repeat(mock_metadata.len as usize);
            std::fs::write(&temp_file, content)?;
            let metadata = std::fs::metadata(&temp_file)?;
            std::fs::remove_file(&temp_file).ok(); // Clean up
            Ok(metadata)
        }
    }
}