# AIOS Roadmap

AIOS is developed in small, testable stages. A stage is complete only when its implementation, tests, and documentation describe the same behavior.

## Stage 0 — Foundation

- [x] Repository structure
- [x] Documentation and contribution guidelines
- [x] License
- [x] GitHub Actions workflow
- [x] Development conventions

**Result:** Open-source project foundation.

## Stage 1 — Desktop Shell

- [x] Tauri 2 application
- [x] React + TypeScript interface
- [x] Navigation and control-plane layout
- [x] Runtime and project views
- [ ] Settings view

**Result:** Desktop application with the primary control-plane surfaces.

## Stage 2 — Core Control Plane

- [x] Rust core modules
- [x] Runtime lifecycle state
- [x] Process-record registration
- [x] Bounded event history
- [x] Tauri command/event communication
- [x] Rust unit tests
- [x] Frontend and Rust CI checks

**Result:** Rust-backed lifecycle state exposed to the desktop UI. This is not an OS process manager.

## Stage 3 — Projects

- [x] Workspace registration and path canonicalization
- [x] Project metadata
- [x] Active-project selection
- [x] Workspace availability refresh
- [x] Git marker detection
- [x] Project state events

**Result:** AIOS can register and track user-selected workspace locations without executing commands or modifying project files.

## Stage 4 — AI Process Lifecycle

- [x] Process records with stable IDs and goals
- [x] Created, running, paused, and stopped states
- [x] Lifecycle transition validation
- [x] Timestamps and bounded event history
- [x] Resource-policy metadata validation

**Result:** AIOS can manage process lifecycle records. These records do not launch agents or operating-system processes. CPU, memory, and runtime settings are not enforced.

## Stage 5 — Model Control Plane

- [x] Local/cloud provider metadata
- [x] Model catalog
- [x] General, planner, coder, and reviewer roles
- [x] Per-role default model selection
- [x] Environment-variable names for credential references
- [x] Model state events and tests

**Result:** AIOS can store provider/model configuration. It does not contact providers or run inference.

## Stage 6 — Persistence and Configuration

- [ ] App-data directory and storage design
- [ ] Versioned schemas and migrations
- [ ] Persist runtime history and project/model configuration
- [ ] Atomic writes and recovery behavior
- [ ] Export/import and reset controls
- [ ] Persistence tests

**Result:** User configuration survives application restarts with an explicit, testable data format.

## Stage 7 — Sandbox and Security Enforcement

- [ ] Capability model and permission policy
- [ ] Per-process authorization
- [ ] Filesystem isolation
- [ ] Process isolation
- [ ] CPU and memory enforcement
- [ ] Network restrictions
- [ ] Security tests and threat model

**Result:** A tested enforcement boundary exists before AI-generated work can access privileged tools.

## Stage 8 — Tools

- [ ] Read-only filesystem inspection
- [ ] Narrowly scoped filesystem writes
- [ ] Terminal execution inside the sandbox
- [ ] Git operations
- [ ] Search and test execution
- [ ] Approval gates and audit records

**Result:** AI Processes can use restricted tools through the Stage 7 security boundary.

## Stage 9 — Verification

- [ ] Build and test orchestration
- [ ] Diff inspection
- [ ] Evidence collection
- [ ] Reproducible verification records
- [ ] Human approval for consequential changes

**Result:** AIOS can distinguish attempted work from verified work.

## Stage 10 — Memory

- [ ] Working memory
- [ ] Project memory
- [ ] Episodic and semantic memory
- [ ] Artifact references
- [ ] Provenance and retention controls

**Result:** AIOS can maintain structured, auditable project context.

## Stage 11 — Orchestration

- [ ] Planner and task graph
- [ ] Multiple AI Processes
- [ ] Model routing
- [ ] Dependencies and scheduling
- [ ] Failure recovery

**Result:** AIOS can coordinate multi-step work across processes.

## Stage 12 — Headless Runtime

- [ ] Separate runtime from desktop
- [ ] Headless operation
- [ ] Stable APIs
- [ ] Runtime configuration
- [ ] Service architecture

**Result:** Core operation no longer depends on the graphical interface.

## Stage 13 — Linux Port

- [ ] Native Linux runtime
- [ ] systemd integration
- [ ] Linux sandboxing and permissions
- [ ] GPU integration
- [ ] Hardware compatibility tests

**Result:** AIOS runs natively as a Linux runtime.

## Stage 14 — AIOS Linux

- [ ] Linux image
- [ ] AIOS services and desktop
- [ ] Boot process
- [ ] Installer
- [ ] Hardware support

**Result:** First bootable AIOS Linux distribution.

## Stage 15 — Hardening

- [ ] Signed releases
- [ ] Immutable system design
- [ ] Atomic updates and rollback
- [ ] Recovery tooling
- [ ] Security review and hardening

**Result:** Production-oriented distribution with a maintainable update and recovery model.
