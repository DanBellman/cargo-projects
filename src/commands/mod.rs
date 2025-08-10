pub mod shared;
pub mod cli;
pub mod list;
pub mod clean;
pub mod clean_watchers;
pub mod refresh;
pub mod scan;
pub mod update;
pub mod watchers;


use crate::types::{ProjectsResult};

pub type CommandResult<T = ()> = ProjectsResult<T>;

pub use list::handle_list_projects;
pub use clean::handle_clean_project;
pub use clean_watchers::handle_clean_watchers;
pub use refresh::handle_refresh_timing;
pub use scan::handle_scan_directory;
pub use update::handle_update_projects;
pub use watchers::handle_list_watchers;

pub use shared::*;