use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const EVENT_HISTORY_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ProcessResourceConfig {
    pub max_cpu_percent: Option<u8>,
    pub max_memory_mb: Option<u64>,
    pub max_runtime_seconds: Option<u64>,
}

impl Default for ProcessResourceConfig {
    fn default() -> Self {
        Self {
            max_cpu_percent: None,
            max_memory_mb: None,
            max_runtime_seconds: None,
        }
    }
}

impl ProcessResourceConfig {
    fn validate(&self) -> Result<(), String> {
        if let Some(cpu) = self.max_cpu_percent {
            if !(1..=100).contains(&cpu) {
                return Err("Max CPU must be between 1 and 100 percent".to_string());
            }
        }
        if let Some(memory) = self.max_memory_mb {
            if memory == 0 {
                return Err("Max memory must be greater than 0 MB".to_string());
            }
        }
        if let Some(runtime) = self.max_runtime_seconds {
            if runtime == 0 {
                return Err("Max runtime must be greater than 0 seconds".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub status: ProcessStatus,
    pub resources: ProcessResourceConfig,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub stopped_at: Option<u64>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeStatus {
    Stopped,
    Running,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub version: &'static str,
    pub status: RuntimeStatus,
    pub started_at: Option<u64>,
    pub uptime_seconds: u64,
    pub process_count: usize,
    pub processes: Vec<ProcessInfo>,
    pub events: Vec<CoreEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvent {
    pub id: u64,
    pub timestamp: u64,
    pub kind: String,
    pub message: String,
}

#[derive(Debug)]
struct RuntimeInner {
    status: RuntimeStatus,
    started_at: Option<u64>,
    next_process_id: u64,
    next_event_id: u64,
    processes: Vec<ProcessInfo>,
    events: VecDeque<CoreEvent>,
}

impl Default for RuntimeInner {
    fn default() -> Self {
        Self {
            status: RuntimeStatus::Stopped,
            started_at: None,
            next_process_id: 1,
            next_event_id: 1,
            processes: Vec::new(),
            events: VecDeque::new(),
        }
    }
}

pub struct RuntimeState {
    inner: Mutex<RuntimeInner>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(RuntimeInner::default()),
        }
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        let inner = self.inner.lock().expect("runtime state lock poisoned");
        snapshot_from(&inner)
    }

    pub fn start(&self) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "runtime state lock poisoned".to_string())?;

        if inner.status == RuntimeStatus::Running {
            return Ok(snapshot_from(&inner));
        }

        let now = now_seconds();
        inner.status = RuntimeStatus::Running;
        inner.started_at = Some(now);
        push_event(&mut inner, "runtime.started", "AIOS runtime started");

        Ok(snapshot_from(&inner))
    }

    pub fn stop(&self) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.inner.lock().map_err(|_| "runtime state lock poisoned".to_string())?;

        if inner.status == RuntimeStatus::Stopped {
            return Ok(snapshot_from(&inner));
        }

        let now = now_seconds();
        inner.status = RuntimeStatus::Stopped;
        inner.started_at = None;

        let process_ids: Vec<String> = inner
            .processes
            .iter()
            .filter(|process| process.status != ProcessStatus::Stopped)
            .map(|process| process.id.clone())
            .collect();

        for process in &mut inner.processes {
            if process.status != ProcessStatus::Stopped {
                process.status = ProcessStatus::Stopped;
                process.stopped_at = Some(now);
                process.updated_at = now;
            }
        }

        push_event(&mut inner, "runtime.stopped", "AIOS runtime stopped");
        for process_id in process_ids {
            push_event(
                &mut inner,
                "process.stopped",
                &format!("Process {process_id} stopped with runtime"),
            );
        }

        Ok(snapshot_from(&inner))
    }

    pub fn create_process(
        &self,
        name: String,
        goal: String,
        resources: ProcessResourceConfig,
    ) -> Result<(ProcessInfo, RuntimeSnapshot), String> {
        let name = name.trim().to_string();
        let goal = goal.trim().to_string();

        if name.is_empty() {
            return Err("Process name cannot be empty".to_string());
        }
        if goal.is_empty() {
            return Err("Process goal cannot be empty".to_string());
        }
        if name.chars().count() > 80 {
            return Err("Process name must be 80 characters or fewer".to_string());
        }
        if goal.chars().count() > 240 {
            return Err("Process goal must be 240 characters or fewer".to_string());
        }
        resources.validate()?;

        let mut inner = self.inner.lock().map_err(|_| "runtime state lock poisoned".to_string())?;

        if inner.status != RuntimeStatus::Running {
            return Err("AIOS runtime is not running".to_string());
        }

        let now = now_seconds();
        let process = ProcessInfo {
            id: format!("proc-{:04}", inner.next_process_id),
            name,
            goal,
            status: ProcessStatus::Created,
            resources,
            created_at: now,
            started_at: None,
            stopped_at: None,
            updated_at: now,
        };
        inner.next_process_id += 1;

        push_event(
            &mut inner,
            "process.created",
            &format!("Created process {}", process.name),
        );
        inner.processes.push(process.clone());

        Ok((process, snapshot_from(&inner)))
    }

    pub fn start_process(&self, id: &str) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.lock_running()?;
        let now = now_seconds();
        let process = find_process_mut(&mut inner, id)?;

        match process.status {
            ProcessStatus::Created | ProcessStatus::Stopped => {
                process.status = ProcessStatus::Running;
                process.started_at = Some(now);
                process.stopped_at = None;
                process.updated_at = now;
                let name = process.name.clone();
                push_event(&mut inner, "process.started", &format!("Started process {name}"));
                Ok(snapshot_from(&inner))
            }
            ProcessStatus::Running => Ok(snapshot_from(&inner)),
            ProcessStatus::Paused => Err(format!("Process '{id}' is paused; resume it instead")),
            ProcessStatus::Failed => Err(format!("Process '{id}' failed and cannot be restarted")),
        }
    }

    pub fn pause_process(&self, id: &str) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.lock_running()?;
        let process = find_process_mut(&mut inner, id)?;

        if process.status != ProcessStatus::Running {
            return Err(format!("Process '{id}' can only be paused while running"));
        }

        let now = now_seconds();
        process.status = ProcessStatus::Paused;
        process.updated_at = now;
        let name = process.name.clone();
        push_event(&mut inner, "process.paused", &format!("Paused process {name}"));

        Ok(snapshot_from(&inner))
    }

    pub fn resume_process(&self, id: &str) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.lock_running()?;
        let process = find_process_mut(&mut inner, id)?;

        if process.status != ProcessStatus::Paused {
            return Err(format!("Process '{id}' can only be resumed while paused"));
        }

        let now = now_seconds();
        process.status = ProcessStatus::Running;
        process.updated_at = now;
        let name = process.name.clone();
        push_event(&mut inner, "process.resumed", &format!("Resumed process {name}"));

        Ok(snapshot_from(&inner))
    }

    pub fn stop_process(&self, id: &str) -> Result<RuntimeSnapshot, String> {
        let mut inner = self.lock_running()?;
        let process = find_process_mut(&mut inner, id)?;

        if process.status == ProcessStatus::Stopped {
            return Ok(snapshot_from(&inner));
        }

        let now = now_seconds();
        process.status = ProcessStatus::Stopped;
        process.stopped_at = Some(now);
        process.updated_at = now;
        let name = process.name.clone();
        push_event(&mut inner, "process.stopped", &format!("Stopped process {name}"));

        Ok(snapshot_from(&inner))
    }

    fn lock_running(&self) -> Result<std::sync::MutexGuard<'_, RuntimeInner>, String> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| "runtime state lock poisoned".to_string())?;
        if inner.status != RuntimeStatus::Running {
            return Err("AIOS runtime is not running".to_string());
        }
        Ok(inner)
    }
}

fn find_process_mut<'a>(inner: &'a mut RuntimeInner, id: &str) -> Result<&'a mut ProcessInfo, String> {
    inner
        .processes
        .iter_mut()
        .find(|process| process.id == id)
        .ok_or_else(|| format!("Process '{id}' not found"))
}

fn snapshot_from(inner: &RuntimeInner) -> RuntimeSnapshot {
    let uptime_seconds = inner
        .started_at
        .map(|started| now_seconds().saturating_sub(started))
        .unwrap_or(0);

    RuntimeSnapshot {
        version: "0.5.0",
        status: inner.status,
        started_at: inner.started_at,
        uptime_seconds,
        process_count: inner.processes.len(),
        processes: inner.processes.clone(),
        events: inner.events.iter().cloned().rev().collect(),
    }
}

fn push_event(inner: &mut RuntimeInner, kind: &str, message: &str) {
    let event = CoreEvent {
        id: inner.next_event_id,
        timestamp: now_seconds(),
        kind: kind.to_string(),
        message: message.to_string(),
    };
    inner.next_event_id += 1;
    inner.events.push_back(event);
    while inner.events.len() > EVENT_HISTORY_LIMIT {
        inner.events.pop_front();
    }
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

    #[test]
    fn runtime_starts_and_stops_cleanly() {
        let state = RuntimeState::new();
        assert_eq!(state.snapshot().status, RuntimeStatus::Stopped);

        let started = state.start().expect("runtime should start");
        assert_eq!(started.status, RuntimeStatus::Running);
        assert_eq!(started.process_count, 0);
        assert_eq!(started.events[0].kind, "runtime.started");

        let stopped = state.stop().expect("runtime should stop");
        assert_eq!(stopped.status, RuntimeStatus::Stopped);
        assert_eq!(stopped.uptime_seconds, 0);
        assert_eq!(stopped.events[0].kind, "runtime.stopped");
    }

    #[test]
    fn processes_require_running_runtime() {
        let state = RuntimeState::new();
        let error = state
            .create_process("Planner".to_string(), "Plan the next task".to_string(), ProcessResourceConfig::default())
            .expect_err("process creation should require a running runtime");
        assert!(error.contains("not running"));

        state.start().expect("runtime should start");
        let (process, snapshot) = state
            .create_process("Planner".to_string(), "Plan the next task".to_string(), ProcessResourceConfig::default())
            .expect("process should be created");
        assert_eq!(process.id, "proc-0001");
        assert_eq!(process.status, ProcessStatus::Created);
        assert_eq!(snapshot.process_count, 1);
        assert_eq!(snapshot.events[0].kind, "process.created");
    }

    #[test]
    fn invalid_processes_are_rejected() {
        let state = RuntimeState::new();
        state.start().expect("runtime should start");

        assert!(state
            .create_process(" ".to_string(), "goal".to_string(), ProcessResourceConfig::default())
            .is_err());
        assert!(state
            .create_process("name".to_string(), " ".to_string(), ProcessResourceConfig::default())
            .is_err());
        assert!(state
            .create_process(
                "name".to_string(),
                "goal".to_string(),
                ProcessResourceConfig {
                    max_cpu_percent: Some(101),
                    ..ProcessResourceConfig::default()
                },
            )
            .is_err());
    }

    #[test]
    fn process_lifecycle_transitions_are_enforced() {
        let state = RuntimeState::new();
        state.start().expect("runtime should start");
        let (process, _) = state
            .create_process("Planner".to_string(), "Plan the next task".to_string(), ProcessResourceConfig::default())
            .expect("process should be created");

        let snapshot = state.start_process(&process.id).expect("process should start");
        assert_eq!(snapshot.processes[0].status, ProcessStatus::Running);

        let snapshot = state.pause_process(&process.id).expect("process should pause");
        assert_eq!(snapshot.processes[0].status, ProcessStatus::Paused);

        let snapshot = state.resume_process(&process.id).expect("process should resume");
        assert_eq!(snapshot.processes[0].status, ProcessStatus::Running);

        let snapshot = state.stop_process(&process.id).expect("process should stop");
        assert_eq!(snapshot.processes[0].status, ProcessStatus::Stopped);
        assert!(snapshot.processes[0].stopped_at.is_some());
        assert_eq!(snapshot.events[0].kind, "process.stopped");
    }

    #[test]
    fn stopping_runtime_stops_active_processes() {
        let state = RuntimeState::new();
        state.start().expect("runtime should start");
        let (process, _) = state
            .create_process("Planner".to_string(), "Plan the next task".to_string(), ProcessResourceConfig::default())
            .expect("process should be created");
        state.start_process(&process.id).expect("process should start");

        let snapshot = state.stop().expect("runtime should stop");
        assert_eq!(snapshot.processes[0].status, ProcessStatus::Stopped);
        assert!(snapshot.processes[0].stopped_at.is_some());
    }
}
