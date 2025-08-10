use std::path::PathBuf;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::types::*;
use crate::repositories::ConfigRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub registry_path: PathBuf,
    pub watcher_registry_path: PathBuf,
    pub max_scan_depth: Option<usize>,
    pub thread_count: Option<usize>,
    pub cache_build_times: bool,
    pub ignore_target_dirs: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        config_dir.push("cargo-projects");

        Self {
            registry_path: config_dir.join("registry.ron"),
            watcher_registry_path: config_dir.join("watchers.ron"),
            max_scan_depth: Some(10),
            thread_count: None,
            cache_build_times: true,
            ignore_target_dirs: true,
        }
    }
}

pub struct ConfigService<C: ConfigRepository> {
    config_repo: Arc<C>,
}

impl<C: ConfigRepository> ConfigService<C> {
    pub fn new(config_repo: Arc<C>) -> Self {
        Self { config_repo }
    }

    pub fn get_config(&self) -> ProjectsResult<AppConfig> {
        self.config_repo.get_config()
    }

    pub fn update_config(&self, config: AppConfig) -> ProjectsResult<()> {
        self.config_repo.update_config(config)
    }

    pub fn get_registry_paths(&self) -> ProjectsResult<(PathBuf, PathBuf)> {
        self.config_repo.get_registry_paths()
    }
}