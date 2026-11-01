use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

const PROJECT_EVENT: &str = "aios:projects";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Ready,
    Missing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInfo {
    pub root: String,
    pub name: String,
    pub vcs: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub workspace: String,
    pub status: ProjectStatus,
    pub repository: Option<RepositoryInfo>,
    pub created_at: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsSnapshot {
    pub version: &'static str,
    pub active_project_id: Option<String>,
    pub project_count: usize,
    pub projects: Vec<ProjectInfo>,
}

#[derive(Debug)]
struct ProjectsInner {
    next_project_id: u64,
    active_project_id: Option<String>,
    projects: Vec<ProjectInfo>,
}

impl Default for ProjectsInner {
    fn default() -> Self {
        Self {
            next_project_id: 1,
            active_project_id: None,
            projects: Vec::new(),
        }
    }
}

pub struct ProjectsState {
    inner: Mutex<ProjectsInner>,
}

impl ProjectsState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(ProjectsInner::default()),
        }
    }

    pub fn snapshot(&self) -> ProjectsSnapshot {
        let inner = self.inner.lock().expect("project state lock poisoned");
        snapshot_from(&inner)
    }

    pub fn register(&self, workspace: String, name: Option<String>) -> Result<ProjectInfo, String> {
        let canonical_workspace = canonicalize_workspace(&workspace)?;
        let repository = detect_repository(&canonical_workspace);
        let project_name = name
            .unwrap_or_else(|| {
                canonical_workspace
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Project")
                    .to_string()
            })
            .trim()
            .to_string();

        if project_name.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
        if project_name.chars().count() > 80 {
            return Err("Project name must be 80 characters or fewer".to_string());
        }

        let mut inner = self.inner.lock().map_err(|_| "project state lock poisoned".to_string())?;

        if inner
            .projects
            .iter()
            .any(|project| project.workspace == canonical_workspace.display().to_string())
        {
            return Err("That workspace is already registered".to_string());
        }

        let id = format!("proj-{:04}", inner.next_project_id);
        inner.next_project_id += 1;
        let active = inner.projects.is_empty();

        let project = ProjectInfo {
            id: id.clone(),
            name: project_name,
            workspace: canonical_workspace.display().to_string(),
            status: ProjectStatus::Ready,
            repository,
            created_at: now_seconds(),
            active,
        };

        if active {
            inner.active_project_id = Some(id);
        }
        inner.projects.push(project.clone());

        Ok(project)
    }

    pub fn remove(&self, id: &str) -> Result<ProjectsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "project state lock poisoned".to_string())?;
        let original_len = inner.projects.len();
        inner.projects.retain(|project| project.id != id);

        if inner.projects.len() == original_len {
            return Err(format!("Project '{id}' not found"));
        }

        if inner.active_project_id.as_deref() == Some(id) {
            inner.active_project_id = inner.projects.first().map(|project| project.id.clone());
        }

        sync_active_flags(&mut inner);
        Ok(snapshot_from(&inner))
    }

    pub fn set_active(&self, id: &str) -> Result<ProjectsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "project state lock poisoned".to_string())?;
        if !inner.projects.iter().any(|project| project.id == id) {
            return Err(format!("Project '{id}' not found"));
        }

        inner.active_project_id = Some(id.to_string());
        sync_active_flags(&mut inner);
        Ok(snapshot_from(&inner))
    }

    pub fn refresh(&self) -> Result<ProjectsSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "project state lock poisoned".to_string())?;

        for project in &mut inner.projects {
            let path = Path::new(&project.workspace);
            if !path.exists() || !path.is_dir() {
                project.status = ProjectStatus::Missing;
                project.repository = None;
                continue;
            }

            project.status = ProjectStatus::Ready;
            project.repository = detect_repository(path);
        }

        sync_active_flags(&mut inner);
        Ok(snapshot_from(&inner))
    }
}

#[tauri::command]
pub fn get_projects(state: State<'_, ProjectsState>) -> ProjectsSnapshot {
    state.snapshot()
}

#[tauri::command]
pub fn register_project(
    workspace: String,
    name: Option<String>,
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<ProjectInfo, String> {
    let project = state.register(workspace, name)?;
    emit_snapshot(&app, &state.snapshot())?;
    Ok(project)
}

#[tauri::command]
pub fn remove_project(
    id: String,
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<ProjectsSnapshot, String> {
    let snapshot = state.remove(&id)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn set_active_project(
    id: String,
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<ProjectsSnapshot, String> {
    let snapshot = state.set_active(&id)?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn refresh_projects(
    state: State<'_, ProjectsState>,
    app: AppHandle,
) -> Result<ProjectsSnapshot, String> {
    let snapshot = state.refresh()?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

fn emit_snapshot(app: &AppHandle, snapshot: &ProjectsSnapshot) -> Result<(), String> {
    app.emit(PROJECT_EVENT, snapshot)
        .map_err(|error| format!("failed to emit project event: {error}"))
}

fn snapshot_from(inner: &ProjectsInner) -> ProjectsSnapshot {
    ProjectsSnapshot {
        version: "0.5.0",
        active_project_id: inner.active_project_id.clone(),
        project_count: inner.projects.len(),
        projects: inner.projects.clone(),
    }
}

fn sync_active_flags(inner: &mut ProjectsInner) {
    let active_id = inner.active_project_id.clone();
    for project in &mut inner.projects {
        project.active = active_id.as_deref() == Some(project.id.as_str());
    }
}

fn canonicalize_workspace(raw: &str) -> Result<PathBuf, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("Workspace path cannot be empty".to_string());
    }
    if value.chars().count() > 500 {
        return Err("Workspace path must be 500 characters or fewer".to_string());
    }

    let path = Path::new(value);
    if !path.exists() {
        return Err(format!("Workspace does not exist: {value}"));
    }
    if !path.is_dir() {
        return Err(format!("Workspace is not a directory: {value}"));
    }

    fs::canonicalize(path).map_err(|error| format!("Failed to resolve workspace: {error}"))
}

fn detect_repository(start: &Path) -> Option<RepositoryInfo> {
    let mut current = Some(start);

    while let Some(path) = current {
        let git_marker = path.join(".git");
        if git_marker.is_dir() || git_marker.is_file() {
            let root = path.to_path_buf();
            let name = root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("repository")
                .to_string();
            return Some(RepositoryInfo {
                root: root.display().to_string(),
                name,
                vcs: "git".to_string(),
            });
        }
        current = path.parent();
    }

    None
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("aios-{name}-{suffix}"))
    }

    #[test]
    fn first_project_becomes_active() {
        let workspace = temp_workspace("first-active");
        fs::create_dir_all(&workspace).expect("workspace should exist");

        let state = ProjectsState::new();
        let project = state
            .register(workspace.to_string_lossy().to_string(), Some("Alpha".to_string()))
            .expect("project should register");

        assert_eq!(project.id, "proj-0001");
        assert!(project.active);
        assert_eq!(state.snapshot().active_project_id.as_deref(), Some("proj-0001"));

        fs::remove_dir_all(workspace).expect("workspace should be removable");
    }

    #[test]
    fn git_repository_is_detected_from_workspace_ancestors() {
        let root = temp_workspace("git-detect");
        let nested = root.join("apps").join("desktop");
        fs::create_dir_all(&nested).expect("nested workspace should exist");
        fs::create_dir_all(root.join(".git")).expect("git marker should exist");

        let state = ProjectsState::new();
        let project = state
            .register(nested.to_string_lossy().to_string(), None)
            .expect("project should register");

        let repository = project.repository.expect("repository should be detected");
        assert_eq!(repository.vcs, "git");
        assert_eq!(repository.root, fs::canonicalize(&root).unwrap().display().to_string());

        fs::remove_dir_all(root).expect("workspace should be removable");
    }

    #[test]
    fn duplicate_workspaces_are_rejected() {
        let workspace = temp_workspace("duplicate");
        fs::create_dir_all(&workspace).expect("workspace should exist");

        let state = ProjectsState::new();
        state
            .register(workspace.to_string_lossy().to_string(), None)
            .expect("first project should register");
        let error = state
            .register(workspace.to_string_lossy().to_string(), None)
            .expect_err("duplicate workspace should fail");

        assert!(error.contains("already registered"));
        fs::remove_dir_all(workspace).expect("workspace should be removable");
    }
}
