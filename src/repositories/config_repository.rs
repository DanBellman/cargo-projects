use std::path::PathBuf;
use std::sync::Arc;
use crate::types::*;
use crate::services::AppConfig;
use crate::infrastructure::FileSystem;
use ron::ser::{to_string_pretty, PrettyConfig};

pub trait ConfigRepository: Send + Sync {
    fn get_config(&self) -> ProjectsResult<AppConfig>;
    fn update_config(&self, config: AppConfig) -> ProjectsResult<()>;
    fn get_registry_paths(&self) -> ProjectsResult<(PathBuf, PathBuf)>;
}

pub struct FileConfigRepository<F: FileSystem> {
    file_system: Arc<F>,
    config_path: PathBuf,
}

impl<F: FileSystem> FileConfigRepository<F> {
    pub fn new(file_system: Arc<F>, config_path: PathBuf) -> Self {
        Self {
            file_system,
            config_path,
        }
    }

    fn load_config(&self) -> ProjectsResult<AppConfig> {
        match self.file_system.exists(&self.config_path) {
            true => self
                .file_system
                .read_to_string(&self.config_path)
                .map_err(Into::into)
                .and_then(|content| ron::from_str(&content).map_err(Into::into)),
            false => Ok(AppConfig::default()),
        }
    }

    fn save_config(&self, config: &AppConfig) -> ProjectsResult<()> {
        to_string_pretty(config, PrettyConfig::default())
            .map_err(Into::into)
            .and_then(|content| self.file_system.write(&self.config_path, &content))
    }
}

impl<F: FileSystem> ConfigRepository for FileConfigRepository<F> {
    fn get_config(&self) -> ProjectsResult<AppConfig> {
        self.load_config()
    }

    fn update_config(&self, config: AppConfig) -> ProjectsResult<()> {
        self.save_config(&config)
    }

    fn get_registry_paths(&self) -> ProjectsResult<(PathBuf, PathBuf)> {
        self.load_config().map(|config| {
            (config.registry_path, config.watcher_registry_path)
        })
    }
}