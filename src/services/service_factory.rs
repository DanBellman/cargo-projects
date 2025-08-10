use std::sync::Arc;
use crate::types::*;
use crate::services::{ProjectService, WatcherService, ConfigService};

/// Factory function for creating a config service with default implementation
pub fn create_default_config_service() -> ProjectsResult<ConfigService<
    crate::repositories::FileConfigRepository<crate::infrastructure::RealFileSystem>,
>> {
    use crate::infrastructure::RealFileSystem;
    use crate::repositories::FileConfigRepository;
    
    let config_dir = dirs::config_dir()
        .ok_or(ProjectsError::ConfigDirectoryNotFound)?
        .join("cargo-projects");

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let config_path = config_dir.join("config.ron");
    let file_system = Arc::new(RealFileSystem);
    
    let config_repo = Arc::new(FileConfigRepository::new(
        file_system,
        config_path,
    ));

    Ok(ConfigService::new(config_repo))
}

/// Factory function for creating a service with default implementations
pub fn create_default_project_service() -> ProjectsResult<ProjectService<
    crate::repositories::FileProjectRepository<crate::infrastructure::RealFileSystem>,
    crate::repositories::FileWatcherRepository<crate::infrastructure::RealFileSystem>,
>> {
    use crate::infrastructure::RealFileSystem;
    use crate::repositories::{FileProjectRepository, FileWatcherRepository};

    let config_service = create_default_config_service()?;
    let (registry_path, watcher_registry_path) = config_service.get_registry_paths()?;
    
    let file_system = Arc::new(RealFileSystem);

    let project_repo = Arc::new(FileProjectRepository::new(
        file_system.clone(),
        registry_path,
        watcher_registry_path.clone(),
    ));
    
    let watcher_repo = Arc::new(FileWatcherRepository::new(
        file_system,
        watcher_registry_path,
    ));

    Ok(ProjectService::new(project_repo, watcher_repo))
}

/// Factory function for creating a watcher service with default implementations
pub fn create_default_watcher_service() -> ProjectsResult<WatcherService<
    crate::repositories::FileWatcherRepository<crate::infrastructure::RealFileSystem>,
>> {
    use crate::infrastructure::RealFileSystem;
    use crate::repositories::FileWatcherRepository;

    let config_service = create_default_config_service()?;
    let (_, watcher_registry_path) = config_service.get_registry_paths()?;
    
    let file_system = Arc::new(RealFileSystem);
    let watcher_repo = Arc::new(FileWatcherRepository::new(
        file_system,
        watcher_registry_path,
    ));

    Ok(WatcherService::new(watcher_repo))
}