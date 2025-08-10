use std::path::Path;
use std::sync::Arc;
use crate::types::*;
use crate::infrastructure::FileSystem;
use ron::ser::{to_string_pretty, PrettyConfig};

pub trait ProjectRepository: Send + Sync {
    fn find_all(&self) -> ProjectsResult<Vec<RustProject>>;
    fn find_by_id(&self, id: ProjectId) -> ProjectsResult<Option<RustProject>>;
    fn find_by_watcher(&self, watcher_name: &WatcherName) -> ProjectsResult<Vec<RustProject>>;
    fn find_by_path(&self, path: &Path) -> ProjectsResult<Option<RustProject>>;
    fn save(&self, project: RustProject) -> ProjectsResult<()>;
    fn save_all(&self, projects: Vec<RustProject>) -> ProjectsResult<()>;
    #[allow(dead_code)]
    fn remove(&self, id: ProjectId) -> ProjectsResult<bool>;
    fn exists(&self, path: &Path) -> ProjectsResult<bool>;
    fn remove_all_projects(&self) -> ProjectsResult<bool>;
}

pub struct FileProjectRepository<F: FileSystem> {
    file_system: Arc<F>,
    registry_path: std::path::PathBuf,
    watcher_registry_path: std::path::PathBuf,
}

impl<F: FileSystem> FileProjectRepository<F> {
    pub fn new(
        file_system: Arc<F>,
        registry_path: std::path::PathBuf,
        watcher_registry_path: std::path::PathBuf,
    ) -> Self {
        Self {
            file_system,
            registry_path,
            watcher_registry_path,
        }
    }

    fn load_registry(&self) -> ProjectsResult<ProjectRegistry> {
        match self.file_system.exists(&self.registry_path) {
            true => self
                .file_system
                .read_to_string(&self.registry_path)
                .map_err(Into::into)
                .and_then(|content| ron::from_str(&content).map_err(Into::into)),
            false => Ok(ProjectRegistry::new()),
        }
    }

    fn save_registry(&self, registry: &ProjectRegistry) -> ProjectsResult<()> {
        to_string_pretty(registry, PrettyConfig::default())
            .map_err(Into::into)
            .and_then(|content| self.file_system.write(&self.registry_path, &content))
    }

    fn load_watcher_registry(&self) -> ProjectsResult<WatcherRegistry> {
        match self.file_system.exists(&self.watcher_registry_path) {
            true => self
                .file_system
                .read_to_string(&self.watcher_registry_path)
                .map_err(Into::into)
                .and_then(|content| ron::from_str(&content).map_err(Into::into)),
            false => Ok(WatcherRegistry::new()),
        }
    }
}

impl<F: FileSystem> ProjectRepository for FileProjectRepository<F> {
    fn find_all(&self) -> ProjectsResult<Vec<RustProject>> {
        self.load_registry()
            .map(|registry| registry.projects.into_values().collect())
    }

    fn find_by_id(&self, id: ProjectId) -> ProjectsResult<Option<RustProject>> {
        self.load_registry()
            .map(|r| r.projects.into_values().find(|p| p.id == id))
    }

    fn find_by_watcher(&self, watcher_name: &WatcherName) -> ProjectsResult<Vec<RustProject>> {
        let registry = self.load_registry()?;
        let watcher_registry = self.load_watcher_registry()?;

        Ok(watcher_registry
            .watchers
            .get(watcher_name.as_str())
            .map_or(Vec::new(), |watcher| {
                registry
                    .projects
                    .into_values()
                    .filter(|p| p.path.starts_with(&watcher.path))
                    .collect()
            }))
    }

    fn find_by_path(&self, path: &Path) -> ProjectsResult<Option<RustProject>> {
        let registry = self.load_registry()?;
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        Ok(registry
            .path_index
            .get(&canonical_path)
            .and_then(|project_path| registry.projects.get(project_path).cloned())
        )

    }

    fn save(&self, project: RustProject) -> ProjectsResult<()> {
        self.load_registry().and_then(| mut registry| {
            registry.add_project(project);
            self.save_registry(&registry)
        })
    }

    fn save_all(&self, projects: Vec<RustProject>) -> ProjectsResult<()> {
        self.load_registry().and_then(|mut registry| {
            projects.into_iter().for_each(|project| registry.add_project(project));
            self.save_registry(&registry)
        })
    }

    fn remove(&self, id: ProjectId) -> ProjectsResult<bool> {
        self.load_registry().and_then(|mut registry| {
            registry.projects
                .iter()
                .find(|(_, project)| project.id == id)
                .map(|(path, _)| path.clone())
                .map_or(Ok(false), |path| {
                    registry.projects.remove(&path);
                    self.save_registry(&registry).map(|_| true)
                })
        })
    }

    fn exists(&self, path: &Path) -> ProjectsResult<bool> {
        self.load_registry()
            .map(|registry| registry.projects.values().any(|p| p.path == path))
    }

    fn remove_all_projects(&self) -> ProjectsResult<bool> {
        self.load_registry().and_then(|mut registry| {
            registry.projects.clear();
            self.save_registry(&registry).map(|_| true)
        })
    }
}