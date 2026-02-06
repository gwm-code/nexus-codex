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
cargo run -- memory set tone "Direct, concise"
cargo run -- memory list

# Phase 3: sandbox shadow run
cargo run -- sandbox --command "ls -la"
cargo run -- sandbox --command "ls -la" --allow-exec --root . --image ubuntu:22.04 --hydrate

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

# Phase 7: audit + benchmarks
cargo run -- audit report
cargo run -- audit mark --performance --security --docs
cargo run -- bench cache --root .
cargo run -- kill-switch --on

# Phase 6: MCP integrations
cargo run -- mcp list
cargo run -- mcp enable GitHub
cargo run -- mcp set-detail GitHub token $GITHUB_TOKEN
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
