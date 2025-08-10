use crate::types::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};


//#TODO Broken. This does not work like I want it to work. It does not work at all :D

pub const fn clear_timing_cache() {
    // Implementation for clearing timing cache
    // Currently a no-op - could clear build time caches in the future
}

pub fn estimate_build_time(path: &PathBuf) -> TimingDuration {
    TimingDuration::new(estimate_build_time_impl(path).unwrap_or(0))
}

fn estimate_build_time_impl(project_path: &PathBuf) -> ProjectsResult<u32> {
    // Check if we have existing timing data
    if let Ok(cached_time) = get_cached_build_time(project_path) {
        return Ok(cached_time);
    }
    
    // Use cargo build --timings to get actual timing data
    let time = run_cargo_timings_estimate(project_path)?;
    
    // Cache the result
    let _ = cache_build_time(project_path, time);
    Ok(time)
}

fn get_cached_build_time(project_path: &Path) -> ProjectsResult<u32> {
    let cache_file = project_path.join("target").join(".build-time-cache");
    if cache_file.exists() {
        let content = fs::read_to_string(cache_file)?;
        Ok(content.trim().parse().map_err(|e| ProjectsError::ParseError { message: format!("Invalid cached build time: {}", e) })?)
    } else {
        Err(ProjectsError::CacheNotFound { message: "No cached build time".to_string() })
    }
}

fn cache_build_time(project_path: &Path, time_seconds: u32) -> ProjectsResult<()> {
    let target_dir = project_path.join("target");
    if target_dir.exists() {
        let cache_file = target_dir.join(".build-time-cache");
        fs::write(cache_file, time_seconds.to_string())?;
    }
    Ok(())
}

fn run_cargo_timings_estimate(project_path: &PathBuf) -> ProjectsResult<u32> {
    // Check if we have existing timing data first
    if let Ok(time) = parse_cargo_timings(project_path) {
        return Ok(time);
    }
    
    // Run cargo check --timings to generate actual timing data
    let output = Command::new("cargo")
        .arg("check")
        .arg("--timings")
        .current_dir(project_path)
        .output()?;
    
    if output.status.success() {
        // Parse the generated timing data
        parse_cargo_timings(project_path).map_err(|_| ProjectsError::ParseError { message: "Could not parse timing data".to_string() })
    } else {
        Err(ProjectsError::BuildError { message: "Check failed".to_string() })
    }
}

fn parse_cargo_timings(project_path: &Path) -> ProjectsResult<u32> {
    let timings_dir = project_path.join("target").join("cargo-timings");
    
    (!timings_dir.exists())
        .then(|| ProjectsError::CacheNotFound { message: "No timings directory found".to_string() })
        .map_or(Ok(()), Err)?;
    
    let mut timing_files: Vec<_> = fs::read_dir(&timings_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            (path.extension()? == "html").then_some(path)
        })
        .collect();
        
    timing_files.is_empty()
        .then(|| ProjectsError::CacheNotFound { message: "No timing files found".to_string() })
        .map_or(Ok(()), Err)?;
    
    timing_files.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    
    timing_files
        .last()
        .ok_or_else(|| ProjectsError::CacheNotFound { message: "No timing files found".to_string() })
        .and_then(|latest_file| fs::read_to_string(latest_file).map_err(Into::into))
        .and_then(|content| {
            // Manual HTML parsing - could use scraper crate instead:
            // let document = scraper::Html::parse_document(&content);
            // let selector = scraper::Selector::parse("td:contains('Total time:') + td").unwrap();
            // document.select(&selector).next()?.text().collect::<String>()
            
            content
                .find("<td>Total time:</td><td>")
                .and_then(|start| {
                    let start = start + 24;
                    content[start..].find("</td>")
                        .map(|end| &content[start..start + end])
                })
                .and_then(parse_time_string)
                .ok_or_else(|| ProjectsError::ParseError { 
                    message: "Could not parse timing data from cargo-timings report".to_string() 
                })
        })
}

fn parse_time_string(time_str: &str) -> Option<u32> {
    humantime::parse_duration(time_str)
        .ok()
        .map(|duration| duration.as_secs() as u32)
}