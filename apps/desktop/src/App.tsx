import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import "./App.css";

type View = "overview" | "projects" | "processes" | "models";
type RuntimeStatus = "stopped" | "running";
type ProcessStatus = "created" | "running" | "paused" | "stopped" | "failed";
type ProjectStatus = "ready" | "missing";
type ProviderKind = "local" | "cloud";
type ModelRole = "general" | "planner" | "coder" | "reviewer";
type ModelProviderInfo = { id: string; name: string; kind: ProviderKind; endpoint: string; authEnv: string | null; enabled: boolean; builtIn: boolean };
type ModelInfo = { id: string; providerId: string; name: string; roles: ModelRole[]; contextWindow: number | null; enabled: boolean };
type ModelSelection = { role: ModelRole; providerId: string; modelId: string };
type ModelsSnapshot = { version: string; providerCount: number; modelCount: number; providers: ModelProviderInfo[]; models: ModelInfo[]; defaults: ModelSelection[] };
type ProviderKind = "local" | "cloud";
type ModelRole = "general" | "planner" | "coder" | "reviewer";
type InputChange = { currentTarget: HTMLInputElement };
type TextareaChange = { currentTarget: HTMLTextAreaElement };

type ProcessResourceConfig = {
  maxCpuPercent: number | null;
  maxMemoryMb: number | null;
  maxRuntimeSeconds: number | null;
};
type ProcessInfo = {
  id: string;
  name: string;
  goal: string;
  status: ProcessStatus;
  resources: ProcessResourceConfig;
  createdAt: number;
  startedAt: number | null;
  stoppedAt: number | null;
  updatedAt: number;
};
type RuntimeEvent = { id: number; timestamp: number; kind: string; message: string };
type RuntimeSnapshot = {
  version: string;
  status: RuntimeStatus;
  startedAt: number | null;
  uptimeSeconds: number;
  processCount: number;
  processes: ProcessInfo[];
  events: RuntimeEvent[];
};
type RepositoryInfo = { root: string; name: string; vcs: string };
type ProjectInfo = {
  id: string; name: string; workspace: string; status: ProjectStatus;
  repository: RepositoryInfo | null; createdAt: number; active: boolean;
};
type ProjectsSnapshot = {
  version: string; activeProjectId: string | null; projectCount: number; projects: ProjectInfo[];
};
type ModelProviderInfo = {
  id: string; name: string; kind: ProviderKind; endpoint: string;
  authEnv: string | null; enabled: boolean; builtIn: boolean;
};
type ModelInfo = {
  id: string; providerId: string; name: string; roles: ModelRole[];
  contextWindow: number | null; enabled: boolean;
};
type ModelSelection = { role: ModelRole; providerId: string; modelId: string };
type ModelsSnapshot = {
  version: string; providerCount: number; modelCount: number;
  providers: ModelProviderInfo[]; models: ModelInfo[]; defaults: ModelSelection[];
};

const RUNTIME_EVENT = "aios:runtime";
const PROJECTS_EVENT = "aios:projects";
const MODELS_EVENT = "aios:models";
const MODELS_EVENT = "aios:models";
const MODEL_ROLES: ModelRole[] = ["general", "planner", "coder", "reviewer"];

const EMPTY_RUNTIME: RuntimeSnapshot = {
  version: "0.4.0", status: "stopped", startedAt: null, uptimeSeconds: 0,
  processCount: 0, processes: [], events: [],
};
const EMPTY_PROJECTS: ProjectsSnapshot = {
  version: "0.4.0", activeProjectId: null, projectCount: 0, projects: [],
};
const EMPTY_MODELS: ModelsSnapshot = {
  version: "0.5.0", providerCount: 0, modelCount: 0, providers: [], models: [], defaults: [],
};

function App() {
  const [view, setView] = useState<View>("overview");
  const [runtime, setRuntime] = useState<RuntimeSnapshot>(EMPTY_RUNTIME);
  const [projects, setProjects] = useState<ProjectsSnapshot>(EMPTY_PROJECTS);
  const [models, setModels] = useState<ModelsSnapshot>(EMPTY_MODELS);
  const [models, setModels] = useState<ModelsSnapshot>(EMPTY_MODELS);
  const [projectName, setProjectName] = useState("");
  const [workspace, setWorkspace] = useState("");
  const [processName, setProcessName] = useState("");
  const [processGoal, setProcessGoal] = useState("");
  const [maxCpuPercent, setMaxCpuPercent] = useState("");
  const [maxMemoryMb, setMaxMemoryMb] = useState("");
  const [maxRuntimeSeconds, setMaxRuntimeSeconds] = useState("");
  const [providerName, setProviderName] = useState("");
  const [providerKind, setProviderKind] = useState<ProviderKind>("local");
  const [providerEndpoint, setProviderEndpoint] = useState("http://127.0.0.1:11434/v1");
  const [providerAuthEnv, setProviderAuthEnv] = useState("");
  const [modelId, setModelId] = useState("");
  const [modelName, setModelName] = useState("");
  const [modelProviderId, setModelProviderId] = useState("");
  const [modelContextWindow, setModelContextWindow] = useState("");
  const [modelRoles, setModelRoles] = useState<ModelRole[]>(["general"]);
  const [selectedRole, setSelectedRole] = useState<ModelRole>("general");
  const [selectedDefault, setSelectedDefault] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let stopRuntimeListener: UnlistenFn | undefined;
    let stopProjectsListener: UnlistenFn | undefined;
    let stopModelsListener: UnlistenFn | undefined;
    let stopModelsListener: UnlistenFn | undefined;
    void (async () => {
      try {
        const [runtimeSnapshot, projectSnapshot, modelSnapshot] = await Promise.all([
          invoke<RuntimeSnapshot>("get_runtime_status"),
          invoke<ProjectsSnapshot>("get_projects"),
          invoke<ModelsSnapshot>("get_models"),
        ]);
        setRuntime(runtimeSnapshot);
        setProjects(projectSnapshot);
        setModels(modelSnapshot);
        if (!modelProviderId && modelSnapshot.providers[0]) setModelProviderId(modelSnapshot.providers[0].id);
        stopRuntimeListener = await listen<RuntimeSnapshot>(RUNTIME_EVENT, (event) => setRuntime(event.payload));
        stopProjectsListener = await listen<ProjectsSnapshot>(PROJECTS_EVENT, (event) => setProjects(event.payload));
        stopModelsListener = await listen<ModelsSnapshot>(MODELS_EVENT, (event) => setModels(event.payload));
      } catch (cause) {
        setError(String(cause));
      }
    })();
    return () => {
      stopRuntimeListener?.();
      stopProjectsListener?.();
      stopModelsListener?.();
    };
  }, []);

  useEffect(() => {
    const existing = models.defaults.find((selection) => selection.role === selectedRole);
    setSelectedDefault(existing ? `${existing.providerId}::${existing.modelId}` : "");
  }, [models.defaults, selectedRole]);

  const runtimeRunning = runtime.status === "running";
  const uptime = useMemo(() => formatDuration(runtime.uptimeSeconds), [runtime.uptimeSeconds]);
  const activeProject = projects.projects.find((project) => project.active) ?? null;

  async function toggleRuntime() {
    setError(""); setBusy(true);
    try {
      const command = runtimeRunning ? "stop_runtime" : "start_runtime";
      setRuntime(await invoke<RuntimeSnapshot>(command));
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  async function handleRegisterProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError("");
    if (!workspace.trim()) { setError("Enter a workspace path first."); return; }
    setBusy(true);
    try {
      await invoke<ProjectInfo>("register_project", { workspace, name: projectName.trim() || null });
      setProjectName(""); setWorkspace("");
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  async function setActiveProject(id: string) {
    setError("");
    try { setProjects(await invoke<ProjectsSnapshot>("set_active_project", { id })); }
    catch (cause) { setError(String(cause)); }
  }

  async function removeProject(id: string) {
    setError("");
    try { setProjects(await invoke<ProjectsSnapshot>("remove_project", { id })); }
    catch (cause) { setError(String(cause)); }
  }

  async function refreshProjects() {
    setError("");
    try { setProjects(await invoke<ProjectsSnapshot>("refresh_projects")); }
    catch (cause) { setError(String(cause)); }
  }

  async function handleCreateProcess(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError("");
    if (!runtimeRunning) { setError("Start the AIOS runtime before creating a process."); return; }
    setBusy(true);
    try {
      await invoke<ProcessInfo>("create_process", {
        name: processName,
        goal: processGoal,
        resources: {
          maxCpuPercent: parseOptionalInteger(maxCpuPercent),
          maxMemoryMb: parseOptionalInteger(maxMemoryMb),
          maxRuntimeSeconds: parseOptionalInteger(maxRuntimeSeconds),
        },
      });
      setProcessName(""); setProcessGoal(""); setMaxCpuPercent(""); setMaxMemoryMb(""); setMaxRuntimeSeconds("");
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  async function transitionProcess(command: "start_process" | "pause_process" | "resume_process" | "stop_process", id: string) {
    setError("");
    try { setRuntime(await invoke<RuntimeSnapshot>(command, { id })); }
    catch (cause) { setError(String(cause)); }
  }

  async function handleRegisterProvider(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError(""); setBusy(true);
    try {
      const created = await invoke<ModelProviderInfo>("register_model_provider", {
        name: providerName, kind: providerKind, endpoint: providerEndpoint, authEnv: providerAuthEnv.trim() || null,
      });
      const snapshot = await invoke<ModelsSnapshot>("get_models");
      setModels(snapshot);
      setModelProviderId(created.id);
      setProviderName(""); setProviderEndpoint(providerKind === "local" ? "http://127.0.0.1:11434/v1" : "https://api.openai.com/v1");
      setProviderAuthEnv("");
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  async function removeProvider(id: string) {
    setError("");
    try {
      const snapshot = await invoke<ModelsSnapshot>("remove_model_provider", { id });
      setModels(snapshot);
      if (modelProviderId === id) setModelProviderId(snapshot.providers[0]?.id ?? "");
    } catch (cause) { setError(String(cause)); }
  }

  async function handleRegisterModel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setError(""); setBusy(true);
    if (!modelProviderId) { setError("Select a provider first."); setBusy(false); return; }
    try {
      await invoke<ModelInfo>("register_model", {
        providerId: modelProviderId,
        modelId,
        name: modelName,
        roles: modelRoles,
        contextWindow: parseOptionalInteger(modelContextWindow),
      });
      setModels(await invoke<ModelsSnapshot>("get_models"));
      setModelId(""); setModelName(""); setModelContextWindow("");
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  async function removeModel(providerId: string, id: string) {
    setError("");
    try { setModels(await invoke<ModelsSnapshot>("remove_model", { providerId, modelId: id })); }
    catch (cause) { setError(String(cause)); }
  }

  async function applyDefaultModel() {
    setError("");
    if (!selectedDefault) {
      try { setModels(await invoke<ModelsSnapshot>("clear_model_default", { role: selectedRole })); }
      catch (cause) { setError(String(cause)); }
      return;
    }
    const [providerId, modelIdValue] = selectedDefault.split("::");
    try {
      setModels(await invoke<ModelsSnapshot>("set_model_default", {
        role: selectedRole, providerId, modelId: modelIdValue,
      }));
    } catch (cause) { setError(String(cause)); }
  }

  function toggleRole(role: ModelRole) {
    setModelRoles((current) =>
      current.includes(role) ? current.filter((item) => item !== role) : [...current, role],
    );
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <div className="brand-mark">A</div>
          <div><div className="eyebrow">AI-NATIVE</div><div className="brand-name">AIOS</div><div className="brand-subtitle">Operating Environment</div></div>
        </div>
        <nav className="nav-list" aria-label="Primary">
          <NavButton active={view === "overview"} onClick={() => setView("overview")} label="Overview" />
          <NavButton active={view === "projects"} onClick={() => setView("projects")} label="Projects" />
          <NavButton active={view === "processes"} onClick={() => setView("processes")} label="Processes" badge={runtime.processCount} />
          <NavButton active={view === "models"} onClick={() => setView("models")} label="Models" badge={models.modelCount} />
          <button className="nav-item nav-disabled" disabled>Settings <span>STAGE 1</span></button>
        </nav>
        <div className="sidebar-footer"><span className={`status-dot ${runtimeRunning ? "online" : "offline"}`} /><span>{runtimeRunning ? "Runtime online" : "Runtime offline"}</span></div>
      </aside>

      <main className="main-panel">
        <header className="topbar">
          <div><div className="breadcrumb">AIOS / {view}</div><h1>{pageTitle(view)}</h1></div>
          <div className="topbar-meta"><span className="version-chip">v{view === "models" ? models.version : runtime.version}</span><button className="runtime-button" onClick={() => void toggleRuntime()} disabled={busy}><span className={`status-dot ${runtimeRunning ? "online" : "offline"}`} />{runtimeRunning ? "Stop runtime" : "Start runtime"}</button></div>
        </header>
        {error && <div className="error-banner">{error}</div>}
        {view === "overview" && <Overview runtime={runtime} runtimeRunning={runtimeRunning} uptime={uptime} activeProject={activeProject} projectCount={projects.projectCount} onProjects={() => setView("projects")} onProcesses={() => setView("processes")} />}
        {view === "projects" && <ProjectsPage projects={projects} projectName={projectName} workspace={workspace} busy={busy} onProjectName={setProjectName} onWorkspace={setWorkspace} onSubmit={handleRegisterProject} onActivate={(id) => void setActiveProject(id)} onRemove={(id) => void removeProject(id)} onRefresh={() => void refreshProjects()} />}
        {view === "models" && <ModelsPage models={models} onRefresh={() => void (async () => setModels(await invoke<ModelsSnapshot>("get_models")))()} onRemoveProvider={(id) => void (async () => { try { setModels(await invoke<ModelsSnapshot>("remove_model_provider", { id })); } catch (cause) { setError(String(cause)); } })()} onRemoveModel={(providerId, modelId) => void (async () => { try { setModels(await invoke<ModelsSnapshot>("remove_model", { providerId, modelId })); } catch (cause) { setError(String(cause)); } })()} onSetDefault={(role, providerId, modelId) => void (async () => { try { setModels(await invoke<ModelsSnapshot>("set_model_default", { role, providerId, modelId })); } catch (cause) { setError(String(cause)); } })()} />}
        {view === "processes" && <ProcessesPage runtime={runtime} runtimeRunning={runtimeRunning} processName={processName} processGoal={processGoal} maxCpuPercent={maxCpuPercent} maxMemoryMb={maxMemoryMb} maxRuntimeSeconds={maxRuntimeSeconds} busy={busy} onName={setProcessName} onGoal={setProcessGoal} onMaxCpuPercent={setMaxCpuPercent} onMaxMemoryMb={setMaxMemoryMb} onMaxRuntimeSeconds={setMaxRuntimeSeconds} onSubmit={handleCreateProcess} onTransition={(command, id) => void transitionProcess(command, id)} />}
        {view === "models" && <ModelsPage
          models={models}
          providerName={providerName}
          providerKind={providerKind}
          providerEndpoint={providerEndpoint}
          providerAuthEnv={providerAuthEnv}
          modelId={modelId}
          modelName={modelName}
          modelProviderId={modelProviderId}
          modelContextWindow={modelContextWindow}
          modelRoles={modelRoles}
          selectedRole={selectedRole}
          selectedDefault={selectedDefault}
          busy={busy}
          onProviderName={setProviderName}
          onProviderKind={(kind) => {
            setProviderKind(kind);
            setProviderEndpoint(kind === "local" ? "http://127.0.0.1:11434/v1" : "https://api.openai.com/v1");
            setProviderAuthEnv(kind === "local" ? "" : "OPENAI_API_KEY");
          }}
          onProviderEndpoint={setProviderEndpoint}
          onProviderAuthEnv={setProviderAuthEnv}
          onModelId={setModelId}
          onModelName={setModelName}
          onModelProviderId={setModelProviderId}
          onModelContextWindow={setModelContextWindow}
          onToggleRole={toggleRole}
          onSelectedRole={setSelectedRole}
          onSelectedDefault={setSelectedDefault}
          onRegisterProvider={handleRegisterProvider}
          onRemoveProvider={(id) => void removeProvider(id)}
          onRegisterModel={handleRegisterModel}
          onRemoveModel={(providerId, id) => void removeModel(providerId, id)}
          onApplyDefault={() => void applyDefaultModel()}
        />}
      </main>
    </div>
  );
}

function NavButton({ active, onClick, label, badge }: { active: boolean; onClick: () => void; label: string; badge?: number }) {
  return <button className={`nav-item ${active ? "active" : ""}`} onClick={onClick}><span>{label}</span>{badge !== undefined && <span className="nav-badge">{badge}</span>}</button>;
}

function Overview({ runtime, runtimeRunning, uptime, activeProject, projectCount, onProjects, onProcesses }: { runtime: RuntimeSnapshot; runtimeRunning: boolean; uptime: string; activeProject: ProjectInfo | null; projectCount: number; onProjects: () => void; onProcesses: () => void }) {
  return <>
    <section className="hero-grid">
      <div className="panel-card hero-card">
        <div className="panel-label">Runtime state</div>
        <div className="runtime-state-row"><span className={`runtime-orb ${runtimeRunning ? "running" : "stopped"}`} /><div><div className="runtime-state-title">{runtimeRunning ? "AIOS runtime active" : "AIOS runtime stopped"}</div><div className="runtime-state-copy">{runtimeRunning ? "The control plane is connected to the Rust core and ready for managed processes." : "Start the core runtime to unlock managed AI processes."}</div></div></div>
        <div className="metric-strip"><Metric label="Uptime" value={uptime} /><Metric label="Processes" value={String(runtime.processCount)} /><Metric label="Projects" value={String(projectCount)} /></div>
      </div>
      <div className="panel-card active-project-card">
        <div className="section-header"><div><div className="panel-label">Active workspace</div><h2>{activeProject?.name ?? "No project selected"}</h2></div><span className={`state-chip ${activeProject?.status ?? "missing"}`}>{activeProject ? activeProject.status : "none"}</span></div>
        <div className="path-value">{activeProject?.workspace ?? "Register a project workspace to give AIOS a context root."}</div>
        {activeProject?.repository ? <RepoInline repository={activeProject.repository} /> : <div className="repo-inline muted"><span className="repo-dot" /><div><strong>No repository detected</strong><span>AIOS will still keep the workspace registered.</span></div></div>}
        <button className="secondary-button" onClick={onProjects}>{projectCount ? "Manage projects" : "Register first project"}</button>
      </div>
    </section>
    <section className="panel-card architecture-card">
      <div className="section-header"><div><div className="panel-label">AIOS architecture</div><h2>Context → Process → Model.</h2></div><button className="text-button" onClick={onProcesses}>Open process manager →</button></div>
      <div className="architecture-flow"><FlowNode title="Desktop" subtitle="React + Tauri" /><span className="flow-arrow">→</span><FlowNode title="Core" subtitle="Rust runtime" /><span className="flow-arrow">→</span><FlowNode title="Projects" subtitle="Workspace boundary" /><span className="flow-arrow">→</span><FlowNode title="Processes" subtitle="Managed work" /><span className="flow-arrow">→</span><FlowNode title="Models" subtitle="Role-based routing" /></div>
      <div className="architecture-note">Models are configuration and routing primitives at Stage 5. AIOS does not yet expose tools or sandboxed model execution.</div>
    </section>
    <section className="content-grid">
      <div className="panel-card compact-card"><div className="panel-label">Core events</div><h2>Runtime activity</h2><div className="activity-list">{runtime.events.slice(0, 5).map((event) => <div className="activity-row" key={event.id}><span className="event-index">#{event.id}</span><div><strong>{event.kind}</strong><span>{event.message}</span></div></div>)}{runtime.events.length === 0 && <div className="empty-state small">No runtime events yet.</div>}</div></div>
      <div className="panel-card compact-card"><div className="panel-label">Model control plane</div><h2>Choose roles, not vendors.</h2><p className="body-copy">AIOS separates provider configuration from role-based model selection. Later stages can consume these defaults without changing the process lifecycle contract.</p><div className="principle-strip"><span>Local</span><span>Cloud</span><span>Planner</span><span>Coder</span><span>Reviewer</span></div></div>
    </section>
  </>;
}

function ProjectsPage({ projects, projectName, workspace, busy, onProjectName, onWorkspace, onSubmit, onActivate, onRemove, onRefresh }: { projects: ProjectsSnapshot; projectName: string; workspace: string; busy: boolean; onProjectName: (value: string) => void; onWorkspace: (value: string) => void; onSubmit: (event: FormEvent<HTMLFormElement>) => void; onActivate: (id: string) => void; onRemove: (id: string) => void; onRefresh: () => void }) {
  return <section className="projects-layout">
    <div className="panel-card registration-card"><div className="panel-label">Project registration</div><h2>Add a workspace</h2><p className="body-copy">Register a local directory. AIOS canonicalizes the path and walks its parents for a Git repository marker.</p>
      <form className="project-form" onSubmit={onSubmit}><label>Workspace path<input value={workspace} onChange={(event: InputChange) => onWorkspace(event.currentTarget.value)} placeholder="C:\Projects\my-app" maxLength={500} /></label><label>Display name <span className="optional">optional</span><input value={projectName} onChange={(event: InputChange) => onProjectName(event.currentTarget.value)} placeholder="Defaults to folder name" maxLength={80} /></label><button className="primary-button" type="submit" disabled={busy}>Register project</button></form><div className="form-footnote">No shell commands are executed during registration.</div>
    </div>
    <div className="panel-card projects-card"><div className="section-header"><div><div className="panel-label">Workspace registry</div><h2>{projects.projectCount} registered</h2></div><button className="secondary-button small-button" onClick={onRefresh}>Refresh state</button></div>
      <div className="project-list">{projects.projects.map((project) => <div className={`project-row ${project.active ? "selected" : ""}`} key={project.id}><div className="project-main"><div className="project-title-row"><strong>{project.name}</strong><span className={`state-chip ${project.status}`}>{project.status}</span>{project.active && <span className="active-pill">ACTIVE</span>}</div><div className="project-path">{project.workspace}</div><div className="project-meta"><span>{project.id}</span>{project.repository ? <span>Git: {project.repository.name}</span> : <span>No repository</span>}</div></div><div className="project-actions">{!project.active && <button className="text-button" onClick={() => onActivate(project.id)}>Set active</button>}<button className="danger-button" onClick={() => onRemove(project.id)}>Remove</button></div></div>)}{projects.projects.length === 0 && <div className="empty-state"><div className="empty-icon">⌂</div><div>No projects registered.</div><span>Add your first workspace to establish AIOS project context.</span></div>}</div>
    </div>
  </section>;
}

function ProcessesPage({ runtime, runtimeRunning, processName, processGoal, maxCpuPercent, maxMemoryMb, maxRuntimeSeconds, busy, onName, onGoal, onMaxCpuPercent, onMaxMemoryMb, onMaxRuntimeSeconds, onSubmit, onTransition }: { runtime: RuntimeSnapshot; runtimeRunning: boolean; processName: string; processGoal: string; maxCpuPercent: string; maxMemoryMb: string; maxRuntimeSeconds: string; busy: boolean; onName: (value: string) => void; onGoal: (value: string) => void; onMaxCpuPercent: (value: string) => void; onMaxMemoryMb: (value: string) => void; onMaxRuntimeSeconds: (value: string) => void; onSubmit: (event: FormEvent<HTMLFormElement>) => void; onTransition: (command: "start_process" | "pause_process" | "resume_process" | "stop_process", id: string) => void }) {
  return <section className="content-grid">
    <div className="panel-card process-card"><div className="section-header"><div><div className="panel-label">Process registration</div><h2>Create a process</h2></div><span className="count-chip">{runtime.processCount}</span></div>
      <form className="process-form" onSubmit={onSubmit}>
        <label>Name<input value={processName} onChange={(event: InputChange) => onName(event.currentTarget.value)} placeholder="e.g. Repository Planner" maxLength={80} /></label>
        <label>Goal<textarea value={processGoal} onChange={(event: TextareaChange) => onGoal(event.currentTarget.value)} placeholder="What should this process accomplish?" maxLength={240} rows={4} /></label>
        <div className="resource-section"><div className="resource-heading">Resource configuration <span>reserved for future enforcement</span></div><div className="resource-grid">
          <label>Max CPU %<input type="number" min="1" max="100" value={maxCpuPercent} onChange={(event: InputChange) => onMaxCpuPercent(event.currentTarget.value)} placeholder="No limit" /></label>
          <label>Max memory MB<input type="number" min="1" value={maxMemoryMb} onChange={(event: InputChange) => onMaxMemoryMb(event.currentTarget.value)} placeholder="No limit" /></label>
          <label>Max runtime sec<input type="number" min="1" value={maxRuntimeSeconds} onChange={(event: InputChange) => onMaxRuntimeSeconds(event.currentTarget.value)} placeholder="No limit" /></label>
        </div></div>
        <button type="submit" className="primary-button" disabled={busy || !runtimeRunning}>{runtimeRunning ? "Create process" : "Start runtime first"}</button>
      </form>
    </div>
    <div className="panel-card process-card"><div className="panel-label">Registered processes</div><h2>Lifecycle manager</h2><div className="process-list">
      {runtime.processes.map((process) => <div className="process-row" key={process.id}><div className="process-main"><div className="process-title-row"><strong>{process.name}</strong><span>{process.id}</span></div><div className="process-goal">{process.goal}</div><div className="process-meta"><span>{formatResources(process.resources)}</span>{process.startedAt ? <span>Started {formatTimestamp(process.startedAt)}</span> : <span>Not started</span>}</div></div><div className="process-actions"><span className={`process-status ${process.status}`}>{process.status}</span>{(process.status === "created" || process.status === "stopped") && <button className="secondary-button small-button" onClick={() => onTransition("start_process", process.id)}>Start</button>}{process.status === "running" && <button className="secondary-button small-button" onClick={() => onTransition("pause_process", process.id)}>Pause</button>}{process.status === "paused" && <button className="secondary-button small-button" onClick={() => onTransition("resume_process", process.id)}>Resume</button>}{(process.status === "running" || process.status === "paused") && <button className="danger-button" onClick={() => onTransition("stop_process", process.id)}>Stop</button>}</div></div>)}
      {runtime.processes.length === 0 && <div className="empty-state small">No AI processes registered yet.</div>}
    </div></div>
  </section>;
}

function ModelsPage({ models, onRefresh, onRemoveProvider, onRemoveModel, onSetDefault }: {
  models: ModelsSnapshot;
  onRefresh: () => void;
  onRemoveProvider: (id: string) => void;
  onRemoveModel: (providerId: string, modelId: string) => void;
  onSetDefault: (role: ModelRole, providerId: string, modelId: string) => void;
}) {
  const [role, setRole] = useState<ModelRole>("general");
  const [providerName, setProviderName] = useState("");
  const [providerKind, setProviderKind] = useState<ProviderKind>("local");
  const [endpoint, setEndpoint] = useState("http://127.0.0.1:11434/v1");
  const [authEnv, setAuthEnv] = useState("");
  const [modelId, setModelId] = useState("");
  const [modelName, setModelName] = useState("");
  const [providerId, setProviderId] = useState(models.providers[0]?.id ?? "");
  const [contextWindow, setContextWindow] = useState("");
  const [roles, setRoles] = useState<ModelRole[]>(["general"]);
  const [selectedDefault, setSelectedDefault] = useState("");
  const [busy, setBusy] = useState(false);
  const [formError, setFormError] = useState("");

  useEffect(() => {
    if (!models.providers.some((provider) => provider.id === providerId)) {
      setProviderId(models.providers[0]?.id ?? "");
    }
  }, [models.providers, providerId]);

  useEffect(() => {
    const current = models.defaults.find((item) => item.role === role);
    setSelectedDefault(current ? `${current.providerId}::${current.modelId}` : "");
  }, [models.defaults, role]);

  const matching = models.models.filter((model) => model.enabled && model.roles.includes(role));
  const providerNameById = new Map(models.providers.map((provider) => [provider.id, provider.name]));

  async function addProvider(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setFormError(""); setBusy(true);
    try {
      const created = await invoke<ModelProviderInfo>("register_model_provider", {
        name: providerName, kind: providerKind, endpoint, authEnv: authEnv.trim() || null,
      });
      setProviderName(""); setAuthEnv("");
      setEndpoint(providerKind === "local" ? "http://127.0.0.1:11434/v1" : "https://api.openai.com/v1");
      setProviderId(created.id);
      onRefresh();
    } catch (cause) { setFormError(String(cause)); }
    finally { setBusy(false); }
  }

  async function addModel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setFormError(""); setBusy(true);
    if (!providerId) { setFormError("Register or select a provider first."); setBusy(false); return; }
    try {
      await invoke<ModelInfo>("register_model", {
        providerId, modelId, name: modelName, roles,
        contextWindow: parseOptionalInteger(contextWindow),
      });
      setModelId(""); setModelName(""); setContextWindow("");
      setRoles(["general"]);
      onRefresh();
    } catch (cause) { setFormError(String(cause)); }
    finally { setBusy(false); }
  }

  function toggleRole(nextRole: ModelRole) {
    setRoles((current) => current.includes(nextRole)
      ? (current.length === 1 ? current : current.filter((item) => item !== nextRole))
      : [...current, nextRole]);
  }

  async function applyDefault() {
    setFormError("");
    if (!selectedDefault) {
      try {
        await invoke<ModelsSnapshot>("clear_model_default", { role });
        onRefresh();
      } catch (cause) { setFormError(String(cause)); }
      return;
    }
    const [selectedProvider, selectedModel] = selectedDefault.split("::");
    onSetDefault(role, selectedProvider, selectedModel);
  }

  return <section className="models-layout">
    <div className="models-left-column">
      <div className="panel-card compact-card">
        <div className="panel-label">Provider registry</div><h2>Add provider</h2>
        <p className="body-copy">Local providers can point at localhost. Cloud providers reference an environment-variable name; AIOS does not store the secret.</p>
        <form className="model-form" onSubmit={addProvider}>
          <label>Name<input value={providerName} onChange={(event: InputChange) => setProviderName(event.currentTarget.value)} placeholder="e.g. Workstation LLM" maxLength={120} /></label>
          <label>Type<select value={providerKind} onChange={(event) => {
            const value = event.currentTarget.value as ProviderKind;
            setProviderKind(value);
            setEndpoint(value === "local" ? "http://127.0.0.1:11434/v1" : "https://api.openai.com/v1");
            setAuthEnv(value === "local" ? "" : "OPENAI_API_KEY");
          }}><option value="local">Local</option><option value="cloud">Cloud</option></select></label>
          <label>Endpoint<input value={endpoint} onChange={(event: InputChange) => setEndpoint(event.currentTarget.value)} placeholder="https://provider.example/v1" maxLength={300} /></label>
          <label>Auth environment variable <span className="optional">optional</span><input value={authEnv} onChange={(event: InputChange) => setAuthEnv(event.currentTarget.value)} placeholder="OPENAI_API_KEY" maxLength={80} /></label>
          <button className="primary-button" type="submit" disabled={busy}>Register provider</button>
        </form>
      </div>

      <div className="panel-card compact-card">
        <div className="panel-label">Model catalog</div><h2>Add model</h2>
        <form className="model-form" onSubmit={addModel}>
          <label>Provider<select value={providerId} onChange={(event) => setProviderId(event.currentTarget.value)}>{models.providers.map((provider) => <option key={provider.id} value={provider.id}>{provider.name}</option>)}</select></label>
          <div className="model-two-col"><label>Model ID<input value={modelId} onChange={(event: InputChange) => setModelId(event.currentTarget.value)} placeholder="qwen2.5-coder:7b" maxLength={120} /></label><label>Display name<input value={modelName} onChange={(event: InputChange) => setModelName(event.currentTarget.value)} placeholder="Qwen Coder 7B" maxLength={120} /></label></div>
          <label>Context window <span className="optional">optional</span><input type="number" min="1" value={contextWindow} onChange={(event: InputChange) => setContextWindow(event.currentTarget.value)} placeholder="32768" /></label>
          <div><div className="model-field-title">Roles</div><div className="role-toggle-row">{["general","planner","coder","reviewer"].map((item) => { const nextRole = item as ModelRole; return <button type="button" key={nextRole} className={`role-toggle ${roles.includes(nextRole) ? "selected" : ""}`} onClick={() => toggleRole(nextRole)}>{nextRole}</button>; })}</div></div>
          <button className="primary-button" type="submit" disabled={busy || !providerId}>Register model</button>
        </form>
      </div>
      {formError && <div className="error-banner">{formError}</div>}
    </div>

    <div className="panel-card compact-card">
      <div className="section-header"><div><div className="panel-label">Role routing</div><h2>Default model by role</h2></div><button className="secondary-button small-button" onClick={applyDefault}>Apply</button></div>
      <div className="model-routing"><label>Role<select value={role} onChange={(event) => setRole(event.currentTarget.value as ModelRole)}>{["general","planner","coder","reviewer"].map((item) => <option key={item} value={item}>{item}</option>)}</select></label><label>Default model<select value={selectedDefault} onChange={(event) => setSelectedDefault(event.currentTarget.value)}><option value="">No default</option>{matching.map((model) => <option key={`${model.providerId}::${model.id}`} value={`${model.providerId}::${model.id}`}>{model.name} · {providerNameById.get(model.providerId) ?? model.providerId}</option>)}</select></label></div>
      <div className="section-divider" />
      <div className="section-header"><div><div className="panel-label">Registered providers</div><h2>{models.providerCount} providers · {models.modelCount} models</h2></div><button className="secondary-button small-button" onClick={onRefresh}>Refresh</button></div>
      <div className="provider-list">{models.providers.map((provider) => <div className="provider-card" key={provider.id}>
        <div className="provider-title-row"><div><strong>{provider.name}</strong><span className={`provider-kind ${provider.kind}`}>{provider.kind}</span>{provider.builtIn && <span className="active-pill">BUILT-IN</span>}</div>{!provider.builtIn && <button className="danger-button" onClick={() => onRemoveProvider(provider.id)}>Remove</button>}</div>
        <div className="provider-endpoint">{provider.endpoint}</div>
        <div className="provider-meta"><span>{provider.authEnv ? `Auth: ${provider.authEnv}` : "No auth variable"}</span></div>
        <div className="provider-model-list">{models.models.filter((model) => model.providerId === provider.id).map((model) => <div className="registered-model-row" key={model.id}><div><strong>{model.name}</strong><span>{model.id} · {model.roles.join(", ")}{model.contextWindow ? ` · ${model.contextWindow.toLocaleString()} ctx` : ""}</span></div><button className="danger-button" onClick={() => onRemoveModel(provider.id, model.id)}>Remove</button></div>)}{models.models.every((model) => model.providerId !== provider.id) && <div className="empty-state small">No models registered.</div>}</div>
      </div>)}</div>
      <div className="model-note">Stage 5 defines the model contract and routing layer. It deliberately stops before model inference, tool access, and sandbox enforcement.</div>
    </div>
  </section>;
}

function RepoInline({ repository }: { repository: RepositoryInfo }) { return <div className="repo-inline"><span className="repo-dot" /><div><strong>{repository.name}</strong><span>{repository.vcs.toUpperCase()} repository detected</span></div></div>; }
function Metric({ label, value }: { label: string; value: string }) { return <div className="metric"><span>{label}</span><strong>{value}</strong></div>; }
function FlowNode({ title, subtitle }: { title: string; subtitle: string }) { return <div className="flow-node"><strong>{title}</strong><span>{subtitle}</span></div>; }
function pageTitle(view: View) { return view === "projects" ? "Project Spaces" : view === "processes" ? "AI Processes" : view === "models" ? "Model Control Plane" : "Control Plane"; }
function parseOptionalInteger(value: string) { const trimmed = value.trim(); if (!trimmed) return null; const parsed = Number.parseInt(trimmed, 10); return Number.isFinite(parsed) ? parsed : null; }
function formatResources(resources: ProcessResourceConfig) {
  const values = [
    resources.maxCpuPercent !== null ? `CPU ≤ ${resources.maxCpuPercent}%` : null,
    resources.maxMemoryMb !== null ? `RAM ≤ ${resources.maxMemoryMb} MB` : null,
    resources.maxRuntimeSeconds !== null ? `Time ≤ ${resources.maxRuntimeSeconds}s` : null,
  ].filter(Boolean);
  return values.length ? values.join(" · ") : "No resource limits configured";
}
function formatTimestamp(timestamp: number) { return new Date(timestamp * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }); }
function formatDuration(totalSeconds: number) { const hours = Math.floor(totalSeconds / 3600); const minutes = Math.floor((totalSeconds % 3600) / 60); const seconds = totalSeconds % 60; if (hours > 0) return `${hours}h ${minutes}m`; if (minutes > 0) return `${minutes}m ${seconds}s`; return `${seconds}s`; }

export default App;
