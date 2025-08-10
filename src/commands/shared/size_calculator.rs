use std::path::Path;

pub fn calculate_directory_size(path: &std::path::PathBuf) -> u64 {
    calculate_project_size(path)
}

pub fn calculate_target_directory_size(path: &std::path::Path) -> u64 {
    calculate_target_size(path)
}

pub fn calculate_project_size(project_path: &std::path::PathBuf) -> u64 {
    use ignore::WalkBuilder;
    use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
    
    let total_size = Arc::new(AtomicU64::new(0));
    
    WalkBuilder::new(project_path)
        .standard_filters(false)
        .git_ignore(false)  // For size calculation, we want all files
        .hidden(false)
        // Use default thread count from ignore crate (automatic scaling)
        .build_parallel()
        .run(|| {
            let total_size = total_size.clone();
            Box::new(move |result| {
                if let Ok(entry) = result {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        if let Ok(metadata) = entry.metadata() {
                            total_size.fetch_add(metadata.len(), Ordering::Relaxed);
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
    
    total_size.load(Ordering::Relaxed)
}

pub fn calculate_target_size(project_path: &Path) -> u64 {
    use ignore::WalkBuilder;
    use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
    
    let target_path = project_path.join("target");
    if !target_path.exists() {
        return 0;
    }
    
    let total_size = Arc::new(AtomicU64::new(0));
    
    WalkBuilder::new(&target_path)
        .standard_filters(false)
        .git_ignore(false)  // For target size, we want all files
        .hidden(false)
        // Use default thread count from ignore crate (automatic scaling)
        .build_parallel()
        .run(|| {
            let total_size = total_size.clone();
            Box::new(move |result| {
                if let Ok(entry) = result {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        if let Ok(metadata) = entry.metadata() {
                            total_size.fetch_add(metadata.len(), Ordering::Relaxed);
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });
    
    total_size.load(Ordering::Relaxed)
}