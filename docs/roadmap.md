# AIOS Roadmap

AIOS is developed incrementally. Each stage should produce a working, testable result before the project moves to the next stage.

## Stage 0 — Foundation

* [x] Repository structure
* [x] Documentation structure
* [x] Contribution guidelines
* [x] License
* [x] GitHub workflow
* [x] Development conventions

**Result:** Professional open-source project foundation.

## Stage 1 — Desktop Shell

* [x] Tauri application
* [x] React + TypeScript interface
* [x] AIOS navigation shell
* [x] Runtime/process interface
* [x] Projects interface
* [ ] Settings

**Result:** AIOS launches as a desktop application with its primary control-plane surfaces.

## Stage 2 — AIOS Core

* [x] Rust core module
* [x] Runtime lifecycle
* [x] Runtime state model
* [x] Process registration
* [x] Event system
* [x] Desktop-to-core communication
* [x] Core unit tests
* [x] Frontend/Rust CI validation

**Result:** The desktop application communicates with a real AIOS runtime.

## Stage 3 — Projects

* [x] Project registration
* [x] Project metadata
* [x] Workspace management
* [x] Repository detection
* [x] Project state

**Result:** AIOS understands the projects it operates on.

## Stage 4 — AI Processes

* [x] Process creation beyond registration
* [x] Goals
* [x] Process state
* [x] Lifecycle management
* [x] Resource configuration
* [x] Process events

**Result:** AIOS can create and manage its first stateful AI Process. Execution remains intentionally deferred until the model and tool stages.

## Stage 5 — Models

* [x] Model provider interface
* [x] Local model support
* [x] Cloud model support
* [x] Model roles
* [x] Model selection

**Result:** AIOS can register local/cloud model providers, maintain a unified model catalog, and route role-based defaults. Inference transport remains behind this interface for the execution layer.

## Stage 6 — Tools

* [ ] Filesystem tools
* [ ] Terminal tools
* [ ] Git tools
* [ ] Search tools
* [ ] Test execution

**Result:** AI Processes can interact with projects.

## Stage 7 — Sandbox

* [ ] Capability system
* [ ] Permission enforcement
* [ ] Process isolation
* [ ] Resource limits
* [ ] Network restrictions

**Result:** AI Processes operate within controlled boundaries.

## Stage 8 — Verification

* [ ] Build verification
* [ ] Test execution
* [ ] Diff inspection
* [ ] Evidence collection
* [ ] Audit records
* [ ] Human approval

**Result:** AIOS can distinguish attempted work from verified work.

## Stage 9 — Memory

* [ ] Working memory
* [ ] Project memory
* [ ] Episodic memory
* [ ] Semantic memory
* [ ] Artifact references
* [ ] Provenance

**Result:** AIOS can maintain structured project context.

## Stage 10 — Orchestration

* [ ] Planner
* [ ] Task graphs
* [ ] Multiple AI Processes
* [ ] Model routing
* [ ] Dependencies
* [ ] Failure recovery

**Result:** AIOS can coordinate complex multi-step tasks.

## Stage 11 — AIOS Runtime

* [ ] Separate runtime from desktop
* [ ] Headless operation
* [ ] Stable APIs
* [ ] Runtime configuration
* [ ] Service architecture

**Result:** AIOS is no longer dependent on its graphical interface.

## Stage 12 — Linux Port

* [ ] Linux runtime
* [ ] systemd integration
* [ ] Linux sandboxing
* [ ] Linux permissions
* [ ] GPU integration

**Result:** AIOS runs natively as a Linux runtime.

## Stage 13 — AIOS Linux

* [ ] Linux image
* [ ] AIOS services
* [ ] AIOS desktop
* [ ] Boot process
* [ ] Installer
* [ ] Hardware support

**Result:** First bootable AIOS Linux distribution.

## Stage 14 — Hardening

* [ ] Signed releases
* [ ] Immutable system
* [ ] Atomic updates
* [ ] Rollback
* [ ] Recovery
* [ ] Security hardening

**Result:** Production-oriented AIOS distribution.
