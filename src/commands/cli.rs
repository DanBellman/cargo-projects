use clap::Parser;
use crate::types::{WatcherName, ProjectId};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
pub enum CargoCli {
    #[command(name = "projects")]
    Projects(ProjectsArgs),
}

// See also `clap_cargo::style::CLAP_STYLING`
pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(clap::Args)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCommand,
}

#[derive(clap::Subcommand)]
pub enum ProjectsCommand {
    List {
        watcher_name: Option<WatcherName>,
    },
    Watchers,
    CleanWatchers,
    Scan {
        #[arg(short, long, default_value = ".")]
        path_to_directory_watched: std::path::PathBuf,
    },
    Clean {
        project_id: ProjectId,
    },
    Watch {
        #[arg(short, long, default_value = ".")]
        project_path: std::path::PathBuf,
        #[arg(short, long)]
        name: Option<WatcherName>,
        #[arg(long)]
        system_wide: bool,
    },
    Update,
    Refresh,
    //ResetRegistry,
    //ResetWatchers,
}