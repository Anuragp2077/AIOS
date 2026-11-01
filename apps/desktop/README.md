# AIOS Desktop

This workspace contains the AIOS desktop control plane: Tauri 2, React 19, TypeScript, and Vite.

## Development

From this directory:

```bash
npm ci
npm run tauri dev
```

Create a frontend production build:

```bash
npm run build
```

The build command runs the TypeScript compiler before Vite.

## Rust checks

From `src-tauri`:

```bash
cargo test --lib
cargo check
```

## Application boundaries

The Rust modules currently manage in-memory runtime lifecycle records, registered workspaces, and model-provider metadata. They do not launch OS processes, execute tools, call model endpoints, or persist state across restarts. Resource settings are metadata only.

For the project overview and full architecture, see the repository [README](../../README.md) and [architecture notes](../../docs/architecture.md).
