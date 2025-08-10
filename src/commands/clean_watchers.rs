use crate::services::create_default_watcher_service;
use super::{CommandResult};


/// Removes all watchers from the watcher_registry.ron. 
pub fn handle_clean_watchers() -> CommandResult {
    create_default_watcher_service()
        .and_then(|service| service.remove_all_watchers().map(|count| {
            println!("Removed {} watchers", count);
        }))
}