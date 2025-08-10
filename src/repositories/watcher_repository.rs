use crate::infrastructure::FileSystem;
use crate::types::*;
use ron::ser::{PrettyConfig, to_string_pretty};
use std::sync::Arc;

pub trait WatcherRepository: Send + Sync {
    fn find_all(&self) -> ProjectsResult<Vec<WatcherConfig>>;
    #[allow(dead_code)]
    fn find_by_name(&self, name: &WatcherName) -> ProjectsResult<Option<WatcherConfig>>;
    fn save(&self, watcher: WatcherConfig) -> ProjectsResult<()>;
    fn remove(&self, name: &WatcherName) -> ProjectsResult<bool>;
    #[allow(dead_code)]
    fn exists(&self, name: &WatcherName) -> ProjectsResult<bool>;
    fn remove_all_watchers(&self) -> ProjectsResult<bool>;
}

pub struct FileWatcherRepository<F: FileSystem> {
    file_system: Arc<F>,
    registry_path: std::path::PathBuf,
}

impl<F: FileSystem> FileWatcherRepository<F> {
    pub fn new(file_system: Arc<F>, registry_path: std::path::PathBuf) -> Self {
        Self {
            file_system,
            registry_path,
        }
    }

    fn load_registry(&self) -> ProjectsResult<WatcherRegistry> {
        match self.file_system.exists(&self.registry_path) {
            true => self
                .file_system
                .read_to_string(&self.registry_path)
                .map_err(Into::into)
                .and_then(|content| ron::from_str(&content).map_err(Into::into)),
            false => Ok(WatcherRegistry::new()),
        }
    }

    fn save_registry(&self, registry: &WatcherRegistry) -> ProjectsResult<()> {
        to_string_pretty(registry, PrettyConfig::default())
            .map_err(Into::into)
            .and_then(|content| self.file_system.write(&self.registry_path, &content))
    }
}

impl<F: FileSystem> WatcherRepository for FileWatcherRepository<F> {
    fn find_all(&self) -> ProjectsResult<Vec<WatcherConfig>> {
        self.load_registry()
            .map(|r| r.watchers.into_values().collect())
    }

    fn find_by_name(&self, name: &WatcherName) -> ProjectsResult<Option<WatcherConfig>> {
        self.load_registry()
            .map(|r| r.watchers.get(name.as_str()).cloned())
    }

    fn save(&self, watcher: WatcherConfig) -> ProjectsResult<()> {
        self.load_registry().and_then(|mut r| {
            r.watchers.insert(watcher.name.to_string(), watcher);
            self.save_registry(&r)
        })
    }

    fn remove(&self, name: &WatcherName) -> ProjectsResult<bool> {
        self.load_registry().and_then(|mut registry| {
            registry
                .watchers
                .remove(name.as_str())
                .map_or(Ok(false), |_| self.save_registry(&registry).map(|_| true))
        })
    }

    fn exists(&self, name: &WatcherName) -> ProjectsResult<bool> {
        self.load_registry()
            .map(|registry| registry.watchers.contains_key(name.as_str()))
    }

    fn remove_all_watchers(&self) -> ProjectsResult<bool> {
        self.load_registry().and_then(|mut registry| {
            registry.watchers.clear();
            self.save_registry(&registry).map(|_| true)
        })
    }
}
