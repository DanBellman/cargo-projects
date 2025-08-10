use byte_unit::{Byte, Unit};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::Infallible,
    fmt,
    fs,
    num::ParseIntError,
    ops::Sub,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectVersion(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct FileSize(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct TimingDuration(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DependencyCount(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatcherName(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectType {
    /// Regular package with [package] section
    #[default]
    Package,
    /// Pure workspace with only [workspace] section
    PureWorkspace,
    /// Workspace root that also has a [package] section (like bevy)
    WorkspaceWithPackage,
    /// Project with malformed Cargo.toml or other issues
    Malformed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustProject {
    pub id: ProjectId,
    pub name: ProjectName,
    pub path: PathBuf,
    pub version: ProjectVersion,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub size_bytes: FileSize,
    #[serde(default)]
    pub target_size_bytes: FileSize,
    pub dependencies_count: DependencyCount,
    #[serde(default)]
    pub estimated_build_time_seconds: TimingDuration,
    #[serde(default)]
    pub project_type: ProjectType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub projects: HashMap<PathBuf, RustProject>,
    #[serde(default)]
    pub path_index: HashMap<PathBuf, PathBuf>,
    pub last_updated: DateTime<Utc>,
    pub next_id: ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub name: WatcherName,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatcherRegistry {
    pub watchers: HashMap<String, WatcherConfig>,
    pub last_updated: DateTime<Utc>,
}

// Implementations for domain types
impl ProjectId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ProjectId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl ProjectName {
    pub const fn new(name: String) -> Self {
        Self(name)
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ProjectVersion {
    pub const fn new(version: String) -> Self {
        Self(version)
    }
}

impl FileSize {
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> u64 {
        self.0
    }
    #[allow(dead_code)]
    pub fn as_human_readable(self) -> String {
        let byte = Byte::from_u64(self.0);
        format!("{byte:#}")
    }

    pub fn as_gb(self) -> f64 {
        let byte = Byte::from_u64(self.0);
        let adjusted = byte.get_adjusted_unit(Unit::GiB);
        adjusted.get_value()
    }
}

impl Sub for FileSize {
    type Output = u64;
    fn sub(self, other: Self) -> u64 {
        self.0.saturating_sub(other.0)
    }
}

impl Sub<u64> for FileSize {
    type Output = u64;
    fn sub(self, other: u64) -> u64 {
        self.0.saturating_sub(other)
    }
}

impl TimingDuration {
    pub const fn new(seconds: u32) -> Self {
        Self(seconds)
    }
    pub const fn seconds(self) -> u32 {
        self.0
    }

    #[allow(dead_code)]
    pub fn as_human_readable(self) -> String {

        if self.0 == 0 {
            "unknown".to_string()
        } else {
            let duration = Duration::from_secs(u64::from(self.0));
            humantime::format_duration(duration).to_string()
        }
    }

    #[allow(dead_code)]
    pub fn as_duration(self) -> Duration {
       Duration::from_secs(u64::from(self.0))
    }
}

impl DependencyCount {
    pub const fn new(count: usize) -> Self {
        Self(count)
    }
}

impl WatcherName {
    pub const fn new(name: String) -> Self {
        Self(name)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WatcherName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for WatcherName {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            path_index: HashMap::new(),
            last_updated: Utc::now(),
            next_id: ProjectId::new(1),
        }
    }

    pub fn add_project(&mut self, mut project: RustProject) {
        project.id = self.next_id;
        self.next_id = self.next_id.next();

        self.path_index
            .retain(|_, project_path| project_path != &project.path);

        self.index_project_paths(&project.path);

        self.projects.insert(project.path.clone(), project);
        self.last_updated = Utc::now();
    }

    fn index_project_paths(&mut self, project_path: &PathBuf) {
        if let Ok(entries) = fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                self.path_index.insert(path.clone(), project_path.clone());

                if path.is_dir() {
                    self.index_directory_recursive(&path, project_path);
                }
            }
        }

        self.path_index
            .insert(project_path.clone(), project_path.clone());
    }

    fn index_directory_recursive(&mut self, dir_path: &PathBuf, project_path: &PathBuf) {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                self.path_index.insert(path.clone(), project_path.clone());

                if path.is_dir() {
                    self.index_directory_recursive(&path, project_path);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn deduplicate(&mut self) {
        let mut deduplicated_projects = HashMap::new();
        let mut next_id = ProjectId::new(1);

        for (path, mut project) in self.projects.drain() {
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());

            if let Entry::Vacant(e) = deduplicated_projects.entry(canonical_path)
            {
                project.id = next_id;
                next_id = next_id.next();
                e.insert(project);
            }
        }

        self.projects = deduplicated_projects;
        self.next_id = next_id;
        self.last_updated = Utc::now();
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &Path) -> crate::types::ProjectsResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let config = ron::ser::PrettyConfig::default();
        let ron_string = ron::ser::to_string_pretty(self, config)?;
        fs::write(path, ron_string)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_from_file(path: &Path) -> crate::types::ProjectsResult<Self> {
        let content = fs::read_to_string(path)?;
        let registry = ron::from_str(&content)?;
        Ok(registry)
    }
}

impl Default for WatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WatcherRegistry {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn add_watcher(&mut self, name: &WatcherName, path: PathBuf) {
        let config = WatcherConfig {
            name: name.clone(),
            path,
            created_at: Utc::now(),
        };
        self.watchers.insert(name.as_str().to_string(), config);
        self.last_updated = Utc::now();
    }

    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &Path) -> crate::types::ProjectsResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let config = ron::ser::PrettyConfig::default();
        let ron_string = ron::ser::to_string_pretty(self, config)?;
        fs::write(path, ron_string)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_from_file(path: &Path) -> crate::types::ProjectsResult<Self> {
        let content = fs::read_to_string(path)?;
        let registry = ron::from_str(&content)?;
        Ok(registry)
    }
}
