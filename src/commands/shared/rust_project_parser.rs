use super::super::CommandResult;
use super::{calculate_project_size, calculate_target_size, estimate_build_time};
use crate::types::*;
use cargo_metadata::MetadataCommand;
use chrono::Utc;
use std::convert::Into;
use std::fs;
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn create_package_project(
    path: &PathBuf,
    project_type: ProjectType,
) -> CommandResult<RustProject> {
    MetadataCommand::new()
        .manifest_path(&path.join("Cargo.toml"))
        .exec()
        .and_then(|metadata| {
            metadata.root_package()
                .ok_or(cargo_metadata::Error::from(IoError::new(
                    ErrorKind::NotFound,
                    "No root package found"
                )))
                .and_then(|package| {
                    let cargo_toml_path = path.join("Cargo.toml");
                    fs::metadata(&cargo_toml_path)
                        .map_err(Into::into)
                        .map(|file_metadata| RustProject {
                            id: ProjectId::new(0),
                            name: ProjectName::new(package.name.to_string()),
                            path: path.clone(),
                            version: ProjectVersion::new(package.version.to_string()),
                            created_at: Utc::now(),
                            last_modified: file_metadata.modified().unwrap_or(SystemTime::now()).into(),
                            size_bytes: FileSize::new(calculate_project_size(path)),
                            target_size_bytes: FileSize::new(calculate_target_size(path)),
                            dependencies_count: DependencyCount::new(package.dependencies.len()),
                            estimated_build_time_seconds: estimate_build_time(path),
                            project_type,
                        })
                })
        })
        .map_err(Into::into)
        .or_else(|_: ProjectsError| Ok(create_malformed_project(path)))
}

pub fn create_workspace_project(path: &PathBuf) -> CommandResult<RustProject> {
    let cargo_toml_path = path.join("Cargo.toml");
    let name = extract_workspace_name(&cargo_toml_path)
        .or_else(|| path.file_name().and_then(|n| n.to_str().map(String::from)))
        .unwrap_or_else(|| "unknown-workspace".to_string());

    fs::metadata(&cargo_toml_path)
        .map_err(Into::into)
        .map(|file_metadata| RustProject {
            id: ProjectId::new(0),
            name: ProjectName::new(name),
            path: path.clone(),
            version: ProjectVersion::new("workspace".to_string()),
            created_at: Utc::now(),
            last_modified: file_metadata.modified().unwrap_or(SystemTime::now()).into(),
            size_bytes: FileSize::new(calculate_project_size(path)),
            target_size_bytes: FileSize::new(calculate_target_size(path)),
            dependencies_count: DependencyCount::new(0),
            estimated_build_time_seconds: estimate_build_time(path),
            project_type: ProjectType::PureWorkspace,
        })
}

pub fn create_malformed_project(path: &PathBuf) -> RustProject {
    let cargo_toml_path = path.join("Cargo.toml");
    let name = extract_package_name(&cargo_toml_path)
        .or_else(|| path.file_name().and_then(|n| n.to_str().map(String::from)))
        .unwrap_or_else(|| "unknown-project".to_string());

    let last_modified = fs::metadata(&cargo_toml_path)
        .and_then(|m| m.modified())
        .map_or_else(|_| Utc::now(), Into::into);

    RustProject {
        id: ProjectId::new(0),
        name: ProjectName::new(name),
        path: path.clone(),
        version: ProjectVersion::new("unknown".to_string()),
        created_at: Utc::now(),
        last_modified,
        size_bytes: FileSize::new(calculate_project_size(path)),
        target_size_bytes: FileSize::new(calculate_target_size(path)),
        dependencies_count: DependencyCount::new(0),
        estimated_build_time_seconds: TimingDuration::new(0),
        project_type: ProjectType::Malformed,
    }
}

/// [workspace.package]
fn extract_workspace_name(cargo_toml_path: &Path) -> Option<String> {
    MetadataCommand::new()
        .manifest_path(cargo_toml_path)
        .exec()
        .ok()?
        .workspace_metadata
        .get("package")?
        .get("name")?
        .as_str()
        .map(|s| s.to_string())
}

/// [package]
fn extract_package_name(cargo_toml_path: &Path) -> Option<String> {
    MetadataCommand::new()
        .manifest_path(cargo_toml_path)
        .exec()
        .ok()?
        .root_package()?
        .name
        .to_string()
        .into()
}
