# Orpheus Coding Agent Blueprint

`orpheus-coding-agent` is the application layer on top of:

- `crates/orpheus` — client, request, and model primitives
- `crates/orpheus-agent` — multi-turn agent orchestration and tool execution

This document records a high-level requirement spec and implementation outline for a future `crates/orpheus-coding-agent` crate.

## Position in the stack

```text
orpheus-coding-agent
  ├─ orpheus-agent
  │   └─ orpheus
  └─ optional future UI/runtime crates
      ├─ orpheus-tui
      └─ orpheus-web-ui
```

## Scope

The crate owns the product/runtime layer needed to turn the existing library crates into a runnable coding assistant.

Functional areas:

1. CLI and runtime startup
2. Session lifecycle and persistence
3. Built-in coding tools
4. Interactive and non-interactive modes
5. Settings and config resolution
6. Prompt, skill, and resource loading
7. Model and provider selection UX
8. Extension and plugin loading
9. Packaging and distribution

## Initial non-goals

These areas can remain out of scope for the first implementation:

- web UI
- remote sync or cloud session storage
- package marketplace or registry
- dynamic loading of untrusted remote code
- full parity with Pi's extension and TUI surface
- multi-user collaboration

## Crate outputs

### Binary

A binary target for end-user execution, for example:

- `orpheus`
- `orpheus-agent`
- `orpheus-code`

### Library

A library API for embedding and tests:

- runtime creation
- session creation and resume
- tool registration
- settings loading
- mode execution

## Proposed module layout

```text
crates/orpheus-coding-agent/
  Cargo.toml
  src/
    lib.rs
    main.rs

    cli/
      mod.rs
      args.rs
      commands.rs

    app/
      mod.rs
      bootstrap.rs
      runtime.rs
      context.rs

    session/
      mod.rs
      manager.rs
      runtime.rs
      storage.rs
      state.rs
      branch.rs
      compaction.rs

    modes/
      mod.rs
      interactive.rs
      print.rs
      json.rs
      rpc.rs

    tools/
      mod.rs
      registry.rs
      bash.rs
      read.rs
      write.rs
      edit.rs
      find.rs
      grep.rs
      ls.rs
      guard.rs

    config/
      mod.rs
      paths.rs
      settings.rs
      merge.rs
      schema.rs

    resources/
      mod.rs
      prompts.rs
      skills.rs
      loader.rs
      templates.rs

    models/
      mod.rs
      registry.rs
      selection.rs
      auth.rs

    extensions/
      mod.rs
      api.rs
      loader.rs
      hooks.rs
      manifest.rs

    ui/
      mod.rs
      theme.rs
      keymap.rs
      components.rs

    output/
      mod.rs
      events.rs
      render.rs
      json.rs

    package/
      mod.rs
      manifest.rs
      manager.rs

    telemetry/
      mod.rs
      tracing.rs
```

A smaller MVP can omit `ui/`, `rpc.rs`, `branch.rs`, `compaction.rs`, and `package/` initially.

## Functional requirements

## 1. CLI and startup

### Requirements

- Parse command-line arguments
- Support interactive and one-shot execution
- Support config, session, and package management commands
- Set working directory and runtime context
- Initialize settings, tools, model selection, and session runtime

### Minimum command surface

- default interactive mode
- `ask <prompt>`
- `resume [session-id]`
- `sessions list`
- `config show`
- `models list`
- `tools list`
- `package list`

### Implementation notes

- `clap` fits the current Rust toolchain already used in examples and tests in this repo
- keep argument parsing separate from runtime construction
- centralize startup in one function such as `run_app(args) -> Result<()>`

## 2. Session lifecycle and persistence

### Requirements

- Create a new session
- Resume an existing session
- Persist conversation history
- Persist tool calls and tool outputs
- Store session metadata:
  - session id
  - cwd
  - created time
  - updated time
  - selected model
  - parent or branch link for future tree support
- Support branching and forking later

### Minimum data model

- `SessionRecord`
- `SessionTurn`
- `SessionEvent`
- `SessionConfigSnapshot`

### Storage requirements

- local filesystem storage
- stable on-disk format
- append-friendly transcript or event log

### Implementation notes

Two storage shapes are viable:

1. event log
   - append each event to JSONL
   - reconstruct state on load
2. snapshot plus transcript
   - store metadata in one file
   - store turns in a second file

The simpler first implementation is snapshot plus transcript.

Suggested layout:

```text
~/.orpheus/sessions/<session-id>/session.json
~/.orpheus/sessions/<session-id>/turns.jsonl
```

Project-local storage can also exist:

```text
<project>/.orpheus/
```

## 3. Agent runtime integration

### Requirements

- Wrap `orpheus-agent::Agent` and `AsyncAgent`
- Convert persisted session state into `Input`
- Run one turn or multi-turn loops
- Capture lower-layer events
- Expose runtime hooks for UI, logging, and persistence

### Existing lower-layer integration points

From `crates/orpheus-agent`:

- `Agent`
- `AsyncAgent`
- `AgentEvent`
- `BeforeToolCall`
- `AfterToolCallContext`
- `ToolExecution`

### Implementation notes

A session-owned runtime type can sit above `orpheus-agent`, for example:

- `CodingSessionRuntime`
- `run_turn()`
- `run_until_idle()`
- `enqueue_user_input()`

This layer owns:

- selected model
- tool registry
- session state
- event fanout

## 4. Built-in coding tools

### Requirements

First-class built-in tools:

- `read`
- `write`
- `edit`
- `bash`

Near-term additions:

- `find`
- `grep`
- `ls`

### Tool behavior requirements

- consistent argument schemas
- structured success and error output
- cwd-awareness
- path normalization
- output truncation
- file size limits
- optional allow and deny restrictions

### Implementation notes

A local tool abstraction can wrap `orpheus-agent::AgentTool`.

Suggested split:

- `tools/registry.rs` — registration and lookup
- one module per tool
- `tools/guard.rs` — output, path, and process guards

Per-tool notes:

- `read`: text and binary detection, truncation, offset and limit support
- `write`: atomic writes where possible
- `edit`: exact replacement API with unique match validation
- `bash`: timeout support and controlled stdout/stderr capture

## 5. Settings and config

### Requirements

- global settings
- project-local settings
- runtime overrides via CLI flags
- merged effective settings view

### Candidate settings fields

| Field | Type | Notes |
|---|---|---|
| `model` | `Option<String>` | default model id |
| `max_turns` | `Option<usize>` | forwarded to agent runtime |
| `tool_execution` | enum | sequential or parallel |
| `session_dir` | `Option<PathBuf>` | storage location |
| `theme` | `Option<String>` | interactive mode |
| `approval_mode` | enum | future tool permissions |
| `disabled_tools` | `Vec<String>` | runtime filtering |
| `extensions` | `Vec<PathBuf>` | enabled extension paths |
| `provider_base_url` | `Option<String>` | client override |
| `output_mode` | enum | text or json |

### Configuration precedence

1. built-in defaults
2. global settings
3. project settings
4. environment variables
5. CLI flags

### Suggested paths

```text
~/.config/orpheus/settings.toml
<project>/.orpheus/settings.toml
```

## 6. Prompt, skill, and resource loading

### Requirements

- Load reusable prompt templates
- Load named skills from markdown or structured files
- Allow project-local resources
- Support bundled built-in resources

### Resource types

- prompts
- skills
- system instructions
- command aliases later

### Suggested directories

```text
~/.config/orpheus/prompts/
~/.config/orpheus/skills/
<project>/.orpheus/prompts/
<project>/.orpheus/skills/
crates/orpheus-coding-agent/resources/
```

### Implementation notes

Start with filesystem-based loading.

Simple initial model:

- prompt = markdown file
- skill = markdown file with frontmatter or section markers

## 7. Model, provider, and auth management

### Requirements

- Choose a default model
- Override model per run or session
- Persist model choice per session
- Read credentials from environment and config
- Leave room for future provider registry growth

### Current lower-layer support

From `crates/orpheus`:

- `Orpheus`
- `AsyncOrpheus`
- request builders with `.model(...)`

### Implementation notes

This layer does not need to reimplement low-level client behavior. It only needs:

- user-facing model selection
- settings-backed defaults
- auth loading and persistence
- optional alias or capability metadata later

A minimal `ModelRegistry` can start with:

- known aliases
- default model id
- capability tags later

## 8. Modes

### Print mode

Requirements:

- execute a prompt
- stream or print final response
- no full-screen UI

Use cases:

- shell use
- scripting
- CI

### JSON mode

Requirements:

- emit structured events
- provide stable machine-readable output

Use cases:

- wrappers
- editors
- automation

### Interactive mode

Requirements:

- prompt input loop
- live assistant output rendering
- visible tool execution messages
- interrupt and cancel support
- session switching later

Implementation notes:

A line-based terminal loop is sufficient for the MVP. Full-screen TUI can be added later.

### RPC mode

Requirements:

- optional
- expose session and run operations over stdio or sockets

Implementation notes:

This can be deferred until after print and JSON modes.

## 9. Extension system

### Requirements

- register extra tools
- register hooks around tool calls and agent events
- register prompt or resource paths
- allow later UI extension points

### Candidate MVP extension surface

| Capability | MVP |
|---|---|
| add tool | yes |
| disable built-in tool | yes |
| before tool hook | yes |
| after tool hook | yes |
| event subscriber | yes |
| add resource directories | yes |
| UI extension | later |
| custom provider | later |

### Implementation notes

Rust makes dynamic loading a different problem than TypeScript. Two workable directions are:

1. compile-time plugin registration
   - extensions are Rust crates or features
   - typed integration
2. external process protocol
   - extension runs as a separate executable
   - communicates over JSON-RPC or stdio

A first implementation can start with compile-time registration or config-driven external tools.

## 10. UI and theming

### Requirements

For an MVP:

- interactive shell loop
- streaming display
- visible tool execution messages

Later additions:

- full TUI
- themes
- keymaps
- selectors

### Implementation notes

Two implementation tracks are possible:

- keep UI inside `orpheus-coding-agent` initially
- split a later `orpheus-tui` crate if the surface grows

## 11. Packaging and assets

### Requirements

- crate binary target
- bundled built-in prompts and skills
- installation and runtime documentation
- release through the existing workspace flow

### Implementation notes

Rust-friendly asset options:

- `include_str!`
- `include_bytes!`

Embedding is sufficient for built-in defaults.

## 12. Observability

### Requirements

- structured logs
- session event logging
- debug mode
- transcript export

### Implementation notes

`tracing` fits this surface.

Possible outputs:

- stderr human logs
- JSON event stream
- persisted session transcript files

## Public API sketch

### Candidate public types

```rust
pub struct CodingAgentRuntime;
pub struct CodingSession;
pub struct SessionManager;
pub struct Settings;
pub struct ToolRegistry;
pub struct ModelRegistry;
```

### Candidate constructors

```rust
CodingAgentRuntime::from_env(...)
CodingAgentRuntime::from_settings(...)
SessionManager::open(...)
CodingSession::new(...)
CodingSession::resume(...)
```

### Candidate operation methods

```rust
run_interactive(...)
run_print(...)
run_json(...)
send_user_message(...)
resume_session(...)
list_sessions(...)
```

The public API can remain smaller than the internal module graph.

## Dependency outline

Likely dependencies:

- `clap` — CLI parsing
- `serde`, `serde_json`, `toml` — config and storage
- `tokio` — async runtime
- `tracing`, `tracing-subscriber` — logs
- `uuid` — session ids
- `chrono` or `time` — timestamps
- `directories` — config and session paths
- `anyhow` or a crate-local error type for the app surface

Internal dependencies:

- `orpheus`
- `orpheus-agent`

## Error model

### Requirements

Separate:

- configuration errors
- session storage errors
- tool execution errors
- model or client errors
- user input or CLI errors

### Candidate error shape

```rust
pub enum CodingAgentError {
    Config(...),
    Session(...),
    Tool(...),
    Client(...),
    Io(...),
}
```

This surface can wrap lower-level `orpheus` and `orpheus-agent` errors.

## MVP definition

A minimal useful first implementation includes:

1. binary crate
2. print mode
3. simple interactive mode
4. session persistence
5. built-in tools:
   - `read`
   - `write`
   - `edit`
   - `bash`
6. settings loading
7. model selection from config or flags
8. transcript export or logging

### Explicitly deferred in MVP

- full TUI
- RPC mode
- extension loading
- package manager
- themes and keybindings
- provider registry UI
- branching session tree

## Phased implementation outline

### Phase 1 — runnable shell

- add crate
- add binary
- wire `orpheus-agent`
- implement print mode
- implement basic interactive loop

### Phase 2 — coding tools

- add tool registry
- add `read`, `write`, `edit`, `bash`
- add truncation, timeout, and path guards

### Phase 3 — persistence

- add session manager
- add transcript storage
- add session resume and list commands

### Phase 4 — configuration

- add global and project settings
- add CLI overrides
- add model and tool configuration

### Phase 5 — richer runtime

- add streaming UX
- add JSON mode
- improve event and render layers

### Phase 6 — extensibility

- add extension API
- add external or custom tools
- add resource loading hooks

### Phase 7 — optional UI split

- move UI concerns into `orpheus-tui` if needed
- add themes, keybindings, and selectors

## Acceptance criteria

The crate is meaningfully present when it can:

- run from a binary command
- send prompts through `orpheus-agent`
- execute built-in coding tools
- persist and resume sessions
- load settings from disk
- stream or print assistant output
- expose structured events for logs or JSON mode

## Smallest viable file plan

A small first cut can start with:

```text
src/main.rs
src/lib.rs
src/cli/mod.rs
src/app/runtime.rs
src/session/manager.rs
src/session/storage.rs
src/tools/mod.rs
src/tools/read.rs
src/tools/write.rs
src/tools/edit.rs
src/tools/bash.rs
src/config/settings.rs
src/modes/print.rs
src/modes/interactive.rs
```

This set is enough to support:

- a runnable app
- a simple session model
- core coding tools
- basic config
