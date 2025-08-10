use std::sync::Arc;
use crate::types::*;
use crate::repositories::{ProjectRepository, WatcherRepository};

pub struct ProjectService<P: ProjectRepository, W: WatcherRepository> {
    project_repo: Arc<P>,
    #[allow(dead_code)]
    watcher_repo: Arc<W>,
}

impl<P: ProjectRepository, W: WatcherRepository> ProjectService<P, W> {
    pub fn new(
        project_repo: Arc<P>,
        watcher_repo: Arc<W>,
    ) -> Self {
        Self {
            project_repo,
            watcher_repo,
        }
    }

    pub fn get_all_projects(&self) -> ProjectsResult<Vec<RustProject>> {
        self.project_repo.find_all()
    }

    pub fn get_project_by_id(&self, id: ProjectId) -> ProjectsResult<Option<RustProject>> {
        self.project_repo.find_by_id(id)
    }

    pub fn get_projects_by_watcher(&self, watcher_name: &WatcherName) -> ProjectsResult<Vec<RustProject>> {
        self.project_repo.find_by_watcher(watcher_name)
    }

    pub fn find_project_by_path(&self, path: &std::path::Path) -> ProjectsResult<Option<RustProject>> {
        self.project_repo.find_by_path(path)
    }

    pub fn find_project_containing_path(&self, path: &std::path::Path) -> ProjectsResult<Option<RustProject>> {
        self.project_repo.find_containing_project(path)
    }

    pub fn add_project(&self, project: RustProject) -> ProjectsResult<()> {
        self.project_repo.save(project)
    }

    pub fn add_projects(&self, projects: Vec<RustProject>) -> ProjectsResult<()> {
        self.project_repo.save_all(projects)
    }

    #[allow(dead_code)]
    pub fn remove_project(&self, id: ProjectId) -> ProjectsResult<bool> {
        self.project_repo.remove(id)
    }

    pub fn project_exists(&self, path: &std::path::Path) -> ProjectsResult<bool> {
        self.project_repo.exists(path)
    }

}
