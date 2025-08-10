use std::sync::Arc;
use crate::types::*;
use crate::repositories::WatcherRepository;


pub struct WatcherService<W: WatcherRepository> {
    watcher_repo: Arc<W>,
}

impl<W: WatcherRepository> WatcherService<W> {
    pub fn new(watcher_repo: Arc<W>) -> Self {
        Self { watcher_repo }
    }

    pub fn get_all_watchers(&self) -> ProjectsResult<Vec<WatcherConfig>> {
        self.watcher_repo.find_all()
    }

    pub fn get_watcher_by_name(&self, name: &WatcherName) -> ProjectsResult<Option<WatcherConfig>> {
        self.watcher_repo.find_by_name(name)
    }

    pub fn add_watcher(&self, watcher: WatcherConfig) -> ProjectsResult<()> {
        self.watcher_repo.save(watcher)
    }

    pub fn remove_watcher(&self, name: &WatcherName) -> ProjectsResult<bool> {
        self.watcher_repo.remove(name)
    }

    #[allow(dead_code)]
    pub fn watcher_exists(&self, name: &WatcherName) -> ProjectsResult<bool> {
        self.watcher_repo.exists(name)
    }

    pub fn remove_all_watchers(&self) -> ProjectsResult<usize> {
        self.watcher_repo.remove_all_watchers().map(|_| 0)
    }
}

