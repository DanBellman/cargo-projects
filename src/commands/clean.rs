use crate::types::*;
use crate::services::create_default_project_service;
use super::{CommandResult};

/// Calls `cargo clean` in the directory in which the specified ProjectId is. Does the same as `cargo clean` because its called.
pub fn handle_clean_project(id: ProjectId) ->
  CommandResult<String> {
      create_default_project_service()?
          .get_project_by_id(id)?
          .ok_or(ProjectsError::ProjectNotFound { id })
          .and_then(|project| {
              std::process::Command::new("cargo")
                  .arg("clean")
                  .current_dir(&project.path)
                  .output()
                  .map_err(Into::into)
                  .and_then(|output| output.status.success()
                      .then_some(project.name.to_string())
                      .ok_or_else(||
  ProjectsError::CargoCommandFailed {
                          stderr:
  String::from_utf8_lossy(&output.stderr).to_string()
                      }))
          })
}