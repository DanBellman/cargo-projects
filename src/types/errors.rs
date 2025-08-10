use thiserror::Error;
use crate::types::types::{ProjectId, WatcherName};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ProjectsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] ron::Error),

    #[error("RON parsing error: {0}")]
    RonSpanned(#[from] ron::error::SpannedError),

    #[error("Cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("Project not found with ID: {id}")]
    ProjectNotFound { id: ProjectId },

    #[error("Watcher not found: {name}")]
    WatcherNotFound { name: WatcherName },

    #[error("Registry file not found at: {path}")]
    RegistryNotFound { path: String },

    #[error("Invalid project name: {name}")]
    InvalidProjectName { name: String },

    #[error("Invalid file size: {size}")]
    InvalidFileSize { size: String },

    #[error("Config directory not found")]
    ConfigDirectoryNotFound,

    #[error("Cargo command failed: {stderr}")]
    CargoCommandFailed { stderr: String },

    #[error("File watching error: {0}")]
    FileWatching(#[from] notify::Error),

    #[error("Build time parsing error: {message}")]
    BuildTimeParsingError { message: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Cache not found: {message}")]
    CacheNotFound { message: String },

    #[error("Build error: {message}")]
    BuildError { message: String },

    #[error("Generic error: {message}")]
    Generic { message: String },
}

pub type ProjectsResult<T> = Result<T, ProjectsError>;

impl From<String> for ProjectsError {
    fn from(message: String) -> Self {
        ProjectsError::Generic { message }
    }
}

impl From<&str> for ProjectsError {
    fn from(message: &str) -> Self {
        ProjectsError::Generic {
            message: message.to_string(),
        }
    }
}