use super::{
    CommandResult, create_malformed_project, create_package_project, create_workspace_project,
};
use crate::services::create_default_project_service;
use crate::types::*;
use log::{info, warn};
use rayon::prelude::*;
use std::path::Path;

use ignore::WalkBuilder;
use std::sync::mpsc;
use std::thread;

pub struct ScanResult {
    pub found_projects: Vec<RustProject>,
    pub added_count: usize,
}

impl From<Vec<RustProject>> for ScanResult {
    fn from(found_projects: Vec<RustProject>) -> Self {
        Self {
            added_count: found_projects.len(),
            found_projects,
        }
    }
}

pub fn handle_scan_directory(path: &Path) -> CommandResult<ScanResult> {
    let service = create_default_project_service()?;
    info!("Scanning for Cargo projects in: {}", path.display());

    let (new_projects, existing): (Vec<_>, Vec<_>) = collect_project_paths(path)?
        .into_iter()
        .map(|project_path| {
            let canonical_path = project_path
                .canonicalize()
                .unwrap_or_else(|_| project_path.clone());
            let exists = service.project_exists(&canonical_path).unwrap_or(false);
            (canonical_path, exists)
        })
        .partition(|(_, exists)| !exists);

    let new_paths: Vec<_> = new_projects
        .into_iter()
        .map(|(path, _)| {
            info!("New project: {}", path.display());
            path
        })
        .collect();

    let existing_count = existing.len();
    existing.into_iter().for_each(|(path, _)| {
        info!("Already tracked: {}", path.display());
    });

    info!(
        " {} new projects, {} already tracked",
        new_paths.len(),
        existing_count
    );

    match new_paths.is_empty() {
        true => Ok(ScanResult {
            found_projects: Vec::new(),
            added_count: 0,
        }),
        false => {
            info!("Processing {} new projects", new_paths.len());
            let found_projects = process_projects_parallel(&new_paths);
            match found_projects.is_empty() {
                true => Ok(found_projects.into()),
                false => {
                    info!("Adding {} to the project registry", found_projects.len());
                    service.add_projects(found_projects.clone())?;
                    info!("Added {} projects!", found_projects.len());
                    Ok(found_projects.into())
                }
            }
        }
    }
}

fn collect_project_paths(path: &Path) -> CommandResult<Vec<std::path::PathBuf>> {
    let (sender, receiver) = mpsc::channel();
    let path = path.to_path_buf();
    let sender_clone = sender.clone();

    let handle = thread::spawn(move || {
        WalkBuilder::new(&path)
            .max_depth(Some(10))
            .standard_filters(false)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(false)
            .hidden(false)
            .threads(
                std::thread::available_parallelism()
                    .map(|n| n.get() * 2)
                    .unwrap_or(8),
            )
            .filter_entry(|entry| {
                let file_name = entry.file_name().to_string_lossy();

                match entry.file_type().is_some_and(|ft| ft.is_dir()) {
                    true => !matches!(
                        file_name.as_ref(),
                        "node_modules"
                            | ".git"
                            | ".svn"
                            | "__pycache__"
                            | ".vscode"
                            | ".idea"
                            | ".venv"
                            | "build"
                            | "dist"
                            | "out"
                            | "target"
                    ),
                    false => true,
                }
            })
            .build_parallel()
            .run(|| {
                let sender = sender_clone.clone();
                Box::new(move |result| {
                    result
                        .ok()
                        .filter(|entry| {
                            entry.file_type().is_some_and(|ft| ft.is_file())
                                && entry.file_name() == "Cargo.toml"
                        })
                        .and_then(|entry| entry.path().parent().map(|p| p.to_path_buf()))
                        .map(|project_path| sender.send(project_path).ok());
                    ignore::WalkState::Continue
                })
            });
    });

    drop(sender);

    let paths: Vec<_> = receiver.into_iter().collect();
    handle.join().map_err(|_| "Thread panicked")?;
    Ok(paths)
}

fn create_rust_project_from_cargo_toml(path: &std::path::PathBuf) -> CommandResult<RustProject> {
    determine_project_type(&path.join("Cargo.toml")).and_then(|project_type| match project_type {
        ProjectType::Package | ProjectType::WorkspaceWithPackage => {
            create_package_project(path, project_type)
        }
        ProjectType::PureWorkspace => create_workspace_project(path),
        ProjectType::Malformed => Ok(create_malformed_project(path)),
    })
}

fn process_projects_parallel(paths: &[std::path::PathBuf]) -> Vec<RustProject> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let processed = Arc::new(AtomicUsize::new(0));
    let total = paths.len();

    paths
        .par_iter()
        .filter_map(|path| {
            let count = processed.fetch_add(1, Ordering::SeqCst) + 1;

            create_rust_project_from_cargo_toml(path).map_or_else(
                |e| {
                    warn!("[{}/{}] Failed: {} - {}", count, total, path.display(), e);
                    None
                },
                |project| {
                    info!(
                        "[{}/{}] Processed: {} ({})",
                        count,
                        total,
                        project.name,
                        path.display()
                    );
                    Some(project)
                },
            )
        })
        .collect()
}

pub fn determine_project_type(cargo_toml_path: &std::path::Path) -> CommandResult<ProjectType> {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(cargo_toml_path)
        .exec()
        .map_err(Into::into)
        .map(|metadata| {
            let has_root_package = metadata.root_package().is_some();
            let has_workspace = !metadata.workspace_members.is_empty();

            match (has_root_package, has_workspace) {
                (true, false) => ProjectType::Package,
                (false, true) => ProjectType::PureWorkspace,
                (true, true) => ProjectType::WorkspaceWithPackage,
                (false, false) => ProjectType::Malformed,
            }
        })
}
