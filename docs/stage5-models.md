# Stage 5 — Models

Stage 5 introduces the AIOS model control plane.

## Provider interface

A provider defines:

- stable provider ID
- local or cloud classification
- endpoint
- enabled/built-in state
- optional environment-variable name for authentication

AIOS never stores the credential value in model state.

Built-in templates:

- **Ollama (Local)** → `http://127.0.0.1:11434/v1`
- **OpenAI-Compatible (Cloud)** → `https://api.openai.com/v1`

Providers are configurable, so other local or cloud endpoints can be registered without changing the core contract.

## Model catalog

Each model records:

- provider ID
- model ID
- display name
- supported roles
- optional context-window metadata
- enabled state

Supported roles:

- general
- planner
- coder
- reviewer

## Routing

AIOS stores one optional default model for each role. A model can only become a role default when that role is declared in its supported-role set.

## Boundary

Stage 5 is deliberately configuration-only. It does not execute inference, shell commands, filesystem mutations, network requests, or external tools.

The next execution layer will consume the provider/model contract while keeping process lifecycle and security boundaries explicit.
