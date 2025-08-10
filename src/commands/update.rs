use super::{
    CommandResult, calculate_directory_size, calculate_target_directory_size, estimate_build_time,
};
use crate::services::create_default_project_service;
use crate::types::*;

pub struct UpdateResult {
    pub updated_projects: Vec<String>,
    pub total_updated: usize,
}

impl From<Vec<String>> for UpdateResult {
    fn from(updated_projects: Vec<String>) -> Self {
        Self {
            total_updated: updated_projects.len(),
            updated_projects,
        }
    }
}

pub fn handle_update_projects() -> CommandResult<UpdateResult> {
    let service = create_default_project_service()?;

    service
        .get_all_projects()?
        .into_iter()
        .map(|project| update_project_metrics(project))
        .filter_map(|(updated, project, name)| updated.then_some((project, name)))
        .map(|(project, name)| service.add_project(project).map(|_| name))
        .collect::<Result<Vec<_>, _>>()
        .map(Into::into)
}


fn update_project_metrics(project: RustProject) -> (bool, RustProject, String) {
    let old_metrics = (
        project.size_bytes,
        project.target_size_bytes,
        project.estimated_build_time_seconds,
    );
    let name = project.name.to_string();

    let new_size = FileSize::new(calculate_directory_size(&project.path));
    let new_target_size = FileSize::new(calculate_target_directory_size(&project.path));
    let new_build_time = estimate_build_time(&project.path);

    let updated_build_time = match new_build_time.seconds() {
        0 => old_metrics.2,
        _ => new_build_time,
    };

    let updated_project = RustProject {
        size_bytes: new_size,
        target_size_bytes: new_target_size,
        estimated_build_time_seconds: updated_build_time,
        ..project
    };

    let has_changes = old_metrics.0.bytes() != new_size.bytes()
        || old_metrics.1.bytes() != new_target_size.bytes()
        || old_metrics.2.seconds() != updated_build_time.seconds();

    (has_changes, updated_project, name)
}
