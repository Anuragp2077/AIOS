# Stage 4 — AI Process Lifecycle

Stage 4 introduces lifecycle records for AI Processes. A record describes a future unit of work; it is not an operating-system process or a running AI agent.

## Process record

Each record contains:

- stable process ID
- display name and goal
- lifecycle state
- creation, start, stop, and update timestamps
- optional resource-policy metadata

Supported transitions:

`created → running → paused → running → stopped`

A stopped record may be started again while the in-app runtime is running. The failed state is reserved for future execution failures and is not currently produced by an executor.

## Commands and events

- `create_process`
- `start_process`
- `pause_process`
- `resume_process`
- `stop_process`

Transitions publish an `aios:runtime` snapshot event.

## Resource metadata

A process can declare a maximum CPU percentage, memory budget in MB, and runtime duration in seconds. Values are validated and stored; they are **not enforced** by the OS.

Enforcement belongs to Stage 7's sandbox and permission boundary. Do not attach terminal, filesystem-write, or network tools to these records before that boundary exists.

## State lifetime

Process records and runtime events currently live in memory and are lost when the desktop application exits. Persistence is planned for Stage 6.

## Deliberate boundary

Stage 4 does not start an AI model, launch an OS process, execute a shell command, mutate files, or make network requests. It supplies the lifecycle contract that later, sandboxed execution components can consume.
