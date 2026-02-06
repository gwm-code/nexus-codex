# Nexus Codex Architecture

## Overview
Nexus Codex is a multi-phase orchestration system that combines a Rust CLI, a local web
dashboard, and a native desktop shell. The system is designed to coordinate safe execution,
context caching, and agent orchestration while keeping user data on-device.

## Core Components
- **CLI Core (`src/main.rs`)**: Entry point for command orchestration, state updates, and provider usage.
- **Cache + Context (`src/cache.rs`, `src/context.rs`)**: Warm cache, diff detection, and handshake/payload generation.
- **Sandbox (`src/sandbox.rs`)**: Shadow-run execution inside Docker with hydration and rollback.
- **Swarm (`src/swarm.rs`)**: Task decomposition, dependency handling, worker scheduling, and self-correction loop.
- **Interface (`src/interface.rs`)**: Local web dashboard backed by shared state snapshots.
- **Desktop UI (`src/desktop.rs`)**: Native control panel for agent activity and system controls.
- **MCP (`src/mcp.rs`)**: Integration configuration layer for external tooling.
- **Vector Store (`src/vector.rs`)**: Local vector search with optional Chroma integration.

## Data Flow
1. The CLI accepts a command and loads configuration.
2. Cache and memory state are loaded to build a status snapshot.
3. Commands are routed to providers, sandbox, swarm, or interface subsystems.
4. Results are stored in local config data under the user config directory.

## Safety Model
Commands are intercepted in dry-run mode. When executed, they are shadow-run in Docker and
hydrated back to the host only on success, with rollback protection if hydration fails.
