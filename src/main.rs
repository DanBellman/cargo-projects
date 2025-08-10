use chrono::Utc;
use clap::Parser;
use log::{error, info};
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::new_debouncer;
use services::{create_default_project_service, service_factory};
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    process::{self},
    sync::mpsc,
    time::Duration,
};

mod commands;
mod infrastructure;
mod output;
mod repositories;
mod services;
mod types;

use commands::cli::{CargoCli, ProjectsCommand};
use commands::*;
use output::output::*;
use types::*;
use crate::commands::scan::determine_project_type;

fn default_error_handler(error: &anyhow::Error, stderr: &mut dyn io::Write) {
    writeln!(stderr, "Error: {}", error).ok();
}

fn main() {
    env_logger::init();
    let result = run();

    match result {
        Err(error) => {
            let stderr = std::io::stderr();
            default_error_handler(&error, &mut stderr.lock());
            process::exit(1);
        }
        Ok(false) => {
            process::exit(1);
        }
        Ok(true) => {
            process::exit(0);
        }
    }
}

fn run() -> anyhow::Result<bool> {
    let CargoCli::Projects(args) = CargoCli::parse();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    match execute_command(args.command) {
        Ok(output) => {
            if let Err(e) = print_result(&mut stdout, &output) {
                let _ = print_error(&mut stderr, &e);
                return Ok(false);
            }
            Ok(true)
        }
        Err(e) => {
            let _ = print_error(&mut stderr, &e);
            Ok(false)
        }
    }
}

//#TODO can it get cleaner?
fn execute_command(command: ProjectsCommand) -> CommandResult<String> {
    match command {
        ProjectsCommand::List { watcher_name } => {
            let project_list_of_watcher = handle_list_projects(watcher_name.as_ref())?;
            Ok(format_project_list(&project_list_of_watcher))
        }
        ProjectsCommand::Watchers => {
            let list_of_watchers = handle_list_watchers()?;
            Ok(format_watcher_list(&list_of_watchers))
        }
        ProjectsCommand::CleanWatchers => {
            handle_clean_watchers()?;
            Ok(format_clean_watchers_result())
        }
        ProjectsCommand::Scan {
            path_to_directory_watched,
        } => {
            let watched_directory = handle_scan_directory(&path_to_directory_watched)?;
            Ok(format_scan_result(&watched_directory))
        }
        ProjectsCommand::Clean { project_id } => {
            let project_name = handle_clean_project(project_id)?;
            Ok(format_clean_result(&project_name))
        }
        ProjectsCommand::Watch {
            project_path,
            name,
            system_wide,
        } => {
            if system_wide {
                return Err(
                    "System-wide monitoring not yet implemented. Use --project-path for now."
                        .into(),
                );
            }

            let watcher_name = name.unwrap_or_else(|| {
                WatcherName::new(
                    project_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unnamed")
                        .to_string(),
                )
            });

            let watcher_config = handle_watch(&project_path, &watcher_name)?;
            Ok(format!(
                "Started watching {} at {}",
                watcher_config.name,
                watcher_config.path.display()
            ))
        }
        ProjectsCommand::Update => {
            let result = handle_update_projects()?;
            Ok(format_update_result(&result))
        }
        ProjectsCommand::Refresh => {
            handle_refresh_timing();
            Ok(format_refresh_result())
        }
    }
}


fn handle_watch(project_path: &std::path::Path, watcher_name: &WatcherName) -> CommandResult<WatcherConfig> {
    use services::service_factory::create_default_watcher_service;
    
    let canonical_path = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
        
    let watcher_config = create_new_watcher_config(watcher_name, &canonical_path);
    
    create_default_watcher_service()
        .and_then(|service| service.add_watcher(watcher_config.clone()))
        .map(|_| watcher_config)
}

fn create_project_from_path(path: &PathBuf) -> CommandResult<RustProject> {

    let cargo_toml_path = path.join("Cargo.toml");
    let project_type = determine_project_type(&cargo_toml_path)?;

    match project_type {
        ProjectType::Package | ProjectType::WorkspaceWithPackage => 
            create_package_project(path, project_type),
        ProjectType::PureWorkspace => 
            create_workspace_project(path),
        ProjectType::Malformed => 
            Ok(create_malformed_project(path)),
    }
}

 fn handle_new_project_detected(project_path: &Path) -> CommandResult {
    create_default_project_service().and_then(|service| {
        service
            .find_project_by_path(project_path)?.ok_or("Project not found".into())
            .and_then(|project| service.add_project(project))
    })
 }

fn register_watcher(name: &WatcherName, path: &Path) -> CommandResult {

    service_factory::create_default_watcher_service().and_then(|service| {
        service.add_watcher(create_new_watcher_config(name, path))
    })
}

fn update_project_size_if_exists(changed_path: &Path) {
    service_factory::create_default_project_service().ok().and_then(|service| {
        service.find_project_containing_path(changed_path).ok()?
            .map(|project| service.add_project(update_project_size_values(project)).ok())
    });
}

//#TODO fix that
fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {    
    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;
    
    debouncer.watch(path.as_ref(), RecursiveMode::Recursive)?;
    info!("Watching for new Rust projects in: {}", path.as_ref().display());

    rx.into_iter()
        .for_each(|result| match result {
            Ok(events) => events
                .into_iter()
                .filter_map(|event| event.paths.first().map(|path| (path.clone(), event.kind)))
                .for_each(|(path, kind)| {
                    match (path.file_name(), kind) {
                        (Some(name), EventKind::Create(_)) if name == OsStr::new("Cargo.toml") => {
                            path.parent()
                                .map(|project_path| {
                                    info!("New Rust project detected: {}", project_path.display());
                                    handle_new_project_detected(project_path)
                                        .map_err(|e| error!("Failed to add new project {}: {}", project_path.display(), e))
                                })
                                .transpose()
                                .ok();
                        }
                        (_, EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)) => {
                            update_project_size_if_exists(&path);
                        }
                        _ => {}
                    }
                }),
            Err(errors) => errors
                .into_iter()
                .for_each(|error| error!("Watch error: {error:?}")),
        });

    Ok(())
}

fn update_project_size_values(project: RustProject) -> RustProject {
    RustProject {
        size_bytes: FileSize::new(calculate_project_size(&project.path)),
        target_size_bytes: FileSize::new(calculate_target_size(&project.path)),
        last_modified: Utc::now(),
        ..project
    }
}

fn create_new_watcher_config(name: &WatcherName, path: &Path) -> WatcherConfig {
    WatcherConfig {
        name: name.clone(),
        path: path.to_path_buf(),
        created_at: Utc::now(),
    }
}
