# Nexus Codex User Guide

## Getting Started
1. Install dependencies for Rust and Docker.
2. Run `cargo run -- config` to view the current configuration.
3. Use `cargo run -- prompt --input "Hello Nexus"` to test provider connectivity.

## Cache and Context
- Warm the cache: `cargo run -- cache-warm --root .`
- Generate handshake metadata: `cargo run -- cache-handshake --root .`
- Build a diff payload: `cargo run -- cache-payload --root . --max-bytes 12000`

## Memory
- Store a preference: `cargo run -- memory set tone "Direct, concise"`
- List entries: `cargo run -- memory list`

## Sandbox
- Shadow-run a command: `cargo run -- sandbox --command "ls -la"`
- Execute tests inside Docker: `cargo run -- sandbox-test --command "cargo test" --root .`

## Swarm
- Plan tasks: `cargo run -- swarm plan "Audit logs\nDraft fixes"`
- Run tasks: `cargo run -- swarm run "Frontend UI updates\nBackend API review\nQA smoke tests"`

## Interface
- Start the dashboard: `cargo run -- serve --addr 127.0.0.1:8888`
- Run daemon mode: `cargo run -- daemon --addr 127.0.0.1:8888 --watch-root .`

## Desktop (AppImage)
- Build a double-clickable AppImage: `./scripts/package-linux.sh`
- Output is saved to `dist/NexusCodex.AppImage` when `appimagetool` is installed.

## Vector Store
- Add a document: `cargo run -- vector add doc-1 "Hello world"`
- Query documents: `cargo run -- vector query "Hello"`

## Audits
- Run a security scan: `cargo run -- audit scan --root .`
- Mark audit items complete: `cargo run -- audit mark --performance --security --docs`
