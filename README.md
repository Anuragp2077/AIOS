# AIOS

**AI-Native Operating Environment**

AIOS explores an operating environment where AI agents can eventually be managed as first-class computational processes rather than ordinary applications. Development starts with a desktop control plane and is intended to progress toward a separately managed runtime and, much later, a Linux-based system.

> **Project status: early prototype — v0.5.0.** AIOS is not currently an operating system, an autonomous agent runtime, or a production-ready tool-execution environment.

## Current implementation

The repository currently contains a Tauri 2 desktop application with a React/TypeScript interface and a Rust control plane.

- **Runtime lifecycle:** start and stop the in-app runtime state; create process records and move them through created, running, paused, and stopped states.
- **Projects:** register an existing local workspace, choose an active project, refresh its availability, and detect a Git repository marker in its parent directories.
- **Models:** configure local or cloud provider metadata, register model entries, and select defaults for general, planner, coder, and reviewer roles.
- **Events:** synchronize runtime, project, and model snapshots with the desktop UI through Tauri events.
- **Validation:** TypeScript production build, Rust unit tests, Rust compile check, and a Tauri permission consistency check are defined in CI.

These are control-plane capabilities, not full execution capabilities.

## Important boundaries

The current implementation deliberately does **not**:

- launch operating-system processes or autonomous AI agents;
- send prompts to model providers or perform model inference;
- execute terminal commands, modify workspace files, or make network requests;
- enforce the CPU, memory, or runtime values stored on process records;
- persist runtime, project, or model state across application restarts;
- provide a sandbox or a general-purpose tool permission system.

Project registration validates and canonicalizes a user-selected directory and checks for a Git marker. It does not crawl or modify project contents. Model credentials are represented only by environment-variable names; secret values are not stored.

Do not treat the current Tauri command allowlist as a sandbox. Real tool execution must wait until persistence, permission enforcement, and process isolation are implemented and tested.

## Architecture

```text
React + TypeScript desktop UI
             │
        Tauri IPC
             │
       Rust control plane
       ├── Runtime lifecycle
       ├── Project registry
       └── Model registry
             │
       In-memory state
```

The intended long-term direction is documented in [docs/architecture.md](docs/architecture.md). The staged implementation plan is in [docs/roadmap.md](docs/roadmap.md).

## Repository structure

```text
.github/
  workflows/           CI workflows
apps/
  desktop/
    src/               React UI
    src-tauri/
      src/core/        Rust runtime, project, and model modules
      capabilities/    Tauri window capabilities
      permissions/     Tauri command permissions
docs/                  Architecture, roadmap, and stage notes
scripts/                Repository validation scripts
```

## Development

Requirements:

- Node.js 22.12 or newer in the Node 22 line
- npm
- Rust stable toolchain and Cargo
- Tauri platform prerequisites for your operating system

Start the desktop application:

```bash
cd apps/desktop
npm ci
npm run tauri dev
```

Build the frontend:

```bash
cd apps/desktop
npm run build
```

Run the Rust checks:

```bash
cd apps/desktop/src-tauri
cargo test --lib
cargo check
```

Run the repository ACL check from the repository root:

```bash
node scripts/check-tauri-acl.mjs
```

See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for native packages required by your platform.

## Development principles

1. **Least privilege:** AI-generated output is untrusted. Privileged actions must be explicit and constrained.
2. **Sandbox before tools:** filesystem, terminal, network, and Git operations must not be exposed before a tested enforcement boundary exists.
3. **Evidence over claims:** builds, tests, diffs, and verification records should support claims that work is complete.
4. **Human control:** consequential operations should have visible approval and auditability.
5. **Local-first:** support local inference where practical, while keeping cloud access explicit.
6. **Incremental delivery:** each roadmap stage should have a testable result and documented security boundary.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md). Persistence and sandboxing are planned before tool execution; the current model registry is configuration-only.

## License

Licensed under the [Apache License 2.0](LICENSE).
