use crate::types::*;
use crate::services::create_default_watcher_service;
use super::CommandResult;

pub struct WatcherListResult {
    pub watchers: Vec<WatcherConfig>,
}

impl From<Vec<WatcherConfig>> for WatcherListResult {
    fn from(watchers: Vec<WatcherConfig>) -> Self {
        Self { watchers }
    }
}

pub fn handle_list_watchers() ->
  CommandResult<WatcherListResult> {
      create_default_watcher_service()?.get_all_watchers().map(Into::into)
}

