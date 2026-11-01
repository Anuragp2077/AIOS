# Stage 5 — Model Control Plane

Stage 5 introduces provider and model configuration. It does not implement inference.

## Provider metadata

A provider records:

- stable provider ID
- local or cloud classification
- HTTP(S) endpoint
- enabled and built-in flags
- optional environment-variable name for authentication

Built-in templates:

- **Ollama (Local):** `http://127.0.0.1:11434/v1`
- **OpenAI-Compatible (Cloud):** `https://api.openai.com/v1`

These are configuration templates only. AIOS does not connect to the endpoints, validate that a server is reachable, or read secret values from the environment.

## Model catalog

Each model records its provider ID, model ID, display name, supported roles, optional context-window metadata, and enabled state.

Supported roles:

- general
- planner
- coder
- reviewer

## Routing

AIOS stores one optional default model for each role. A model can become a role default only when that role appears in the model's supported-role list.

## State and security boundary

Provider records, model records, and role defaults are stored in memory and reset when the application exits. Persistence is planned for Stage 6.

No API key value is stored. No inference, terminal execution, filesystem mutation, network request, or external tool is performed by this stage. Any future provider transport must be implemented behind explicit permissions and the Stage 7 security boundary.
