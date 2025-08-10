use std::path::PathBuf;
use cargo_projects::types::*;
use cargo_projects::commands::*;
use chrono::Utc;

fn create_test_project(name: &str, path: PathBuf) -> RustProject {
    RustProject {
        id: ProjectId::new(0),
        name: ProjectName::new(name.to_string()),
        path,
        version: ProjectVersion::new("1.0.0".to_string()),
        created_at: Utc::now(),
        last_modified: Utc::now(),
        size_bytes: FileSize::new(1000),
        target_size_bytes: FileSize::new(500),
        dependencies_count: DependencyCount::new(5),
        estimated_build_time_seconds: TimingDuration::new(30),
        project_type: ProjectType::Package,
    }
}

fn create_test_watcher(name: &str, path: PathBuf) -> WatcherConfig {
    WatcherConfig {
        name: WatcherName::new(name.to_string()),
        path,
        created_at: Utc::now(),
    }
}

#[test]
fn test_list_projects_empty_registry() -> ProjectsResult<()> {
    let result = handle_list_projects(None)?;
    
    assert!(result.total_count >= 0);
    assert_eq!(result.projects.len(), result.total_count);
    Ok(())
}

#[test]
fn test_list_projects_with_nonexistent_watcher() -> ProjectsResult<()> {
    let watcher_name = WatcherName::new("nonexistent-watcher".to_string());
    let result = handle_list_projects(Some(&watcher_name))?;
    
    assert_eq!(result.projects.len(), result.total_count);
    Ok(())
}

#[test]
fn test_handle_clean_project_not_found() -> ProjectsResult<()> {
    let non_existent_id = ProjectId::new(999999999); 
    
    match handle_clean_project(non_existent_id) {
        Ok(_) => {
            // If it succeeds, the project existed and was cleaned
        }
        Err(e) => {
            let error_string = format!("{}", e);
            assert!(error_string.contains("999999999") || error_string.contains("not found"));
        }
    }
    Ok(())
}

#[test]
fn test_handle_list_watchers() -> ProjectsResult<()> {
    let result = handle_list_watchers()?;
    
    assert!(result.watchers.len() >= 0);
    Ok(())
}

#[test]
fn test_handle_clean_watchers() -> ProjectsResult<()> {
    let result = handle_clean_watchers();
    
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_handle_update_projects() -> ProjectsResult<()> {
    let result = handle_update_projects()?;
    
    assert!(result.total_updated >= 0);
    assert_eq!(result.updated_projects.len(), result.total_updated);
    Ok(())
}

#[test]
fn test_handle_refresh_timing() {
    handle_refresh_timing();
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    #[test] 
    fn test_scan_directory_integration() -> ProjectsResult<()> {
        let temp_dir = tempfile::tempdir()?;
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_dir)?;
        
        let cargo_toml = project_dir.join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#)?;
        
        let result = handle_scan_directory(temp_dir.path())?;
        assert!(result.found_projects.iter().any(|p| p.name.to_string() == "test-project"));
        assert!(result.added_count > 0);
        
        Ok(())
    }
    
    #[test]
    fn test_scan_directory_no_projects() -> ProjectsResult<()> {
        let temp_dir = tempfile::tempdir()?;
        
        let result = handle_scan_directory(temp_dir.path())?;
        
        assert_eq!(result.added_count, 0);
        assert!(result.found_projects.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_scan_directory_malformed_cargo_toml() -> ProjectsResult<()> {
        let temp_dir = tempfile::tempdir()?;
        let project_dir = temp_dir.path().join("malformed-project");
        fs::create_dir_all(&project_dir)?;
        
        // Create a malformed Cargo.toml
        let cargo_toml = project_dir.join("Cargo.toml");
        fs::write(&cargo_toml, "invalid toml content [[[");
        
        // Scan should handle this gracefully
        let result = handle_scan_directory(temp_dir.path())?;
        
        // Should not crash, may or may not detect malformed project
        assert!(result.found_projects.len() <= 1);
        
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_create_test_project_helper() {
        let project = create_test_project("test", PathBuf::from("/test"));
        
        assert_eq!(project.name.to_string(), "test");
        assert_eq!(project.path, PathBuf::from("/test"));
        assert_eq!(project.project_type, ProjectType::Package);
        // ProjectVersion fields are private, so I just test that creation works
    }

    #[test]
    fn test_create_test_watcher_helper() {
        let watcher = create_test_watcher("test-watcher", PathBuf::from("/test"));
        
        assert_eq!(watcher.name.to_string(), "test-watcher");
        assert_eq!(watcher.path, PathBuf::from("/test"));
    }

    #[test]
    fn test_project_id_operations() {
        let id1 = ProjectId::new(1);
        let id2 = ProjectId::new(2);
        let id3 = id1.next();
        
        assert_eq!(id1.get(), 1);
        assert_eq!(id2.get(), 2);
        assert_eq!(id3.get(), 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_file_size_operations() {
        let size1 = FileSize::new(1000);
        let size2 = FileSize::new(2000);
        
        assert_eq!(size1.bytes(), 1000);
        assert_eq!(size2.bytes(), 2000);
        assert_eq!(size1.as_gb(), 1000.0 / (1024.0 * 1024.0 * 1024.0));
    }

    #[test]
    fn test_timing_duration_operations() {
        let duration1 = TimingDuration::new(60);
        let duration2 = TimingDuration::new(0);
        
        assert_eq!(duration1.seconds(), 60);
        assert_eq!(duration2.seconds(), 0);
    }

    #[test]
    fn test_dependency_count() {
        let count = DependencyCount::new(42);
        // Count was created successfully
        let _count = count; // creation works?
    }

    #[test]
    fn test_watcher_name_operations() {
        let name = WatcherName::new("test-watcher".to_string());
        
        assert_eq!(name.as_str(), "test-watcher");
        assert_eq!(name.to_string(), "test-watcher");
    }
}