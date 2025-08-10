use crate::commands::{
    list::ProjectListResult, scan::ScanResult, update::UpdateResult, watchers::WatcherListResult,
};
use std::io::{self, Write};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Columns},
};

#[derive(Tabled)]
struct ProjectTableRow {
    #[tabled(rename = "ID")]
    id: u32,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Size (GB)")]
    size: String,
    #[tabled(rename = "Cache (GB)")]
    cache: String,
    #[tabled(rename = "Check Time")]
    check_time: String,
}

pub fn format_project_list(result: &ProjectListResult) -> String {
    match result.projects.is_empty() {
        true => "No projects found.".to_string(),
        false => format!(
            "{}\nTotal: {} projects",
            Table::new(
                &result
                    .projects
                    .iter()
                    .map(|p| ProjectTableRow {
                        id: p.id.get(),
                        name: p.name.to_string(),
                        size: format!("{:.3}", p.size_bytes.as_gb()),
                        cache: format!("{:.3}", p.target_size_bytes.as_gb()),
                        check_time: match p.estimated_build_time_seconds.seconds() {
                            0 => "Unknown".to_string(),
                            s => format_build_time(s),
                        },
                    })
                    .collect::<Vec<_>>()
            )
            .with(Style::modern())
            .with(Modify::new(Columns::new(0..1)).with(Alignment::right()))
            .with(Modify::new(Columns::new(2..5)).with(Alignment::right())),
            result.total_count
        ),
    }
}

pub fn format_watcher_list(result: &WatcherListResult) -> String {
    match result.watchers.is_empty() {
        true => "No watchers configured.".to_string(),
        false => format!(
            "{:<20} {:<50} {:<20}\n{:<20} {:<50} 
  {:<20}\n{}",
            "Name",
            "Path",
            "Created",
            "----",
            "----",
            "-------",
            result
                .watchers
                .iter()
                .map(|w| format!(
                    "{:<20} {:<50} {:<20}",
                    w.name.as_str(),
                    w.path.display(),
                    w.created_at.format("%Y-%m-%d %H:%M")
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

pub fn format_scan_result(result: &ScanResult) -> String {
    match result.added_count {
        0 => "No new projects found.".to_string(),
        _ => format!(
            "Found and added {} new projects:\n{}",
            result.added_count,
            result
                .found_projects
                .iter()
                .map(|p| format!("  • {} ({})", p.name, p.path.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

pub fn format_update_result(result: &UpdateResult) -> String {
    match result.total_updated {
        0 => "All projects are up to date.".to_string(),
        _ => format!(
            "Updated {} projects:\n{}",
            result.total_updated,
            result
                .updated_projects
                .iter()
                .map(|name| format!("  • {name}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

pub fn format_clean_result(project_name: &str) -> String {
    format!("Cleaned project: {project_name}")
}

pub fn format_clean_watchers_result() -> String {
    "All watchers cleared".to_string()
}

pub fn format_refresh_result() -> String {
    "Timing cache cleared. Run 'update' to refresh timing data.".to_string()
}

fn format_build_time(seconds: u32) -> String {
      match seconds {
          0 => "Unknown".to_string(),
          s if s < 60 => format!("{s}s"),
          s if s < 3600 => {
              let (mins, secs) = (s / 60, s % 60);
              match secs {
                  0 => format!("{mins}m"),
                  _ => format!("{mins}m{secs}s")
              }
          },
          s => {
              let (hours, mins) = (s / 3600, (s % 3600) / 60);
              match mins {
                  0 => format!("{hours}h"),
                  _ => format!("{hours}h{mins}m")
              }
          }
      }
}

// Output functions that handle the actual printing
pub fn print_result<W: Write>(writer: &mut W, content: &str) -> io::Result<()> {
    writeln!(writer, "{content}")
}

pub fn print_error<W: Write>(writer: &mut W, error: &dyn std::error::Error) -> io::Result<()> {
    writeln!(writer, "Error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_project_list_empty() {
        let result = ProjectListResult {
            projects: vec![],
            total_count: 0,
        };
        assert_eq!(format_project_list(&result), "No projects found.");
    }

    #[test]
    fn test_format_build_time() {
        assert_eq!(format_build_time(0), "Unknown");
        assert_eq!(format_build_time(30), "30s");
        assert_eq!(format_build_time(60), "1m");
        assert_eq!(format_build_time(90), "1m30s");
        assert_eq!(format_build_time(3600), "1h");
        assert_eq!(format_build_time(3660), "1h1m");
    }
}
