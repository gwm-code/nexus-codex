# nexus-codex

Nexus Codex is a planning repository for Project Nexus. This update builds a
multi-phase CLI foundation that implements the roadmap through Phase 5 with
working safety, memory, cache, swarm scaffolding, a local interface server, and
a native desktop UI shell. It also introduces Phase 6 self-healing scanning and
Phase 7 audit scaffolding.

## CLI app

```bash
cargo run -- config
cargo run -- prompt --input "Hello Nexus"
cargo run -- prompt --input "Hello Nexus" --live
echo "ls -la" | cargo run -- run
cargo run -- tui

# Phase 2: cache + memory
cargo run -- cache-warm --root .
cargo run -- cache-diff --root .
cargo run -- cache-handshake --root .
cargo run -- cache-payload --root . --max-bytes 12000
cargo run -- memory set tone "Direct, concise"
cargo run -- memory list

# Phase 3: sandbox shadow run
cargo run -- sandbox --command "ls -la"
cargo run -- sandbox --command "ls -la" --allow-exec --root . --image ubuntu:22.04 --hydrate
cargo run -- sandbox-test --command "cargo test" --root .

# Phase 4: swarm orchestration scaffolding
cargo run -- swarm plan "Scan repo\nSummarize risks\nDraft fixes"
cargo run -- swarm run "Scan repo\nSummarize risks\nDraft fixes"

# Phase 5: interface server
cargo run -- serve --addr 127.0.0.1:8888
cargo run -- daemon --addr 127.0.0.1:8888 --log-path dev.log --watch-root .

# Phase 5: rich CLI diff viewer
cargo run -- diff-view --root .

# Phase 6: self-healing scan
cargo run -- heal scan --log-path dev.log
cargo run -- heal list
#
# Notifications
cargo run -- notify list
cargo run -- notify clear

# Phase 7: audit + benchmarks
cargo run -- audit report
cargo run -- audit scan --root .
cargo run -- audit mark --performance --security --docs
cargo run -- bench cache --root .
cargo run -- bench vector --docs 500
cargo run -- kill-switch --on

# Phase 6: MCP integrations
cargo run -- mcp list
cargo run -- mcp enable GitHub
cargo run -- mcp set-detail GitHub token $GITHUB_TOKEN

# Vector store
cargo run -- vector add doc-1 "Hello world"
cargo run -- vector query "Hello"
```

The dashboard is available at http://127.0.0.1:8888 with live status, diff review,
incidents, audit status, kill switch, and PWA install support.

Provider configuration is stored in `nexus.toml` and supports per-provider API keys
plus optional model/base URL overrides (Gemini/OpenRouter/OpenCode/Claude).

## Desktop app

```bash
cargo run --bin desktop
```

The desktop UI surfaces live status, cache controls, memory vault management,
and the interface server toggle for dashboard integrations. It also includes
self-healing log scanning, a kill-switch toggle, incident review, and MCP integration toggles.

## Linux desktop packaging (no terminal required)

Build a self-contained AppImage for double-click execution:

```bash
./scripts/package-linux.sh
```

If `appimagetool` is available, the script produces `dist/NexusCodex.AppImage`.
Otherwise, it emits `dist/nexus-desktop.AppDir.tar.gz`, which can be extracted and
run by double-clicking `AppRun` in a file manager after creating one or using the
desktop entry inside the AppDir.【F:scripts/package-linux.sh†L1-L35】

Key design goals:
- Safety-first command interception in dry-run mode.
- Provider abstraction ready for Gemini and other backends.
- Cache + diff engine for large repos and context handshakes.
- Long-term memory vault for preferences across sessions.
- Shadow-run sandbox entry point (execution gated by flag).
- Swarm planning/execution scaffolding for parallel agents.
- Interface server for dashboard integrations.
- Native desktop UI for day-to-day orchestration.
- Self-healing incident scan, audit scaffolding, and kill switch.

This foundation is designed to grow into the hardening, self-healing, and
release phases described in `roadmap.md`.

## Documentation
- [Architecture](docs/architecture.md)
- [User Guide](docs/user-guide.md)
- [API Reference](docs/api.md)
- [Beta Program](docs/beta-program.md)
- [Release Pipeline](docs/release-pipeline.md)
