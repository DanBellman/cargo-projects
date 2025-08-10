use super::CommandResult;
use crate::services::create_default_project_service;
use crate::types::*;

pub struct ProjectListResult {
    pub projects: Vec<RustProject>,
    pub total_count: usize,
}

pub fn handle_list_projects(
    watcher_name: Option<&WatcherName>,
) -> CommandResult<ProjectListResult> {
    create_default_project_service()
        .and_then(|service| {
            watcher_name.map_or(service.get_all_projects(), |name| {
                service.get_projects_by_watcher(name)
            })
        })
        .map(|mut projects| {
            projects.sort_by_key(|p| p.id);
            ProjectListResult {
                total_count: projects.len(),
                projects,
            }
        })
}
