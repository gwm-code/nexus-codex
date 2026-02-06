# Nexus Codex User Guide

## Getting Started
1. Install dependencies for Rust and Docker.
2. Run `cargo run -- config` to view the current configuration.
3. Use `cargo run -- prompt --input "Hello Nexus"` to test provider connectivity.

## Easy Install (Non-Technical, Ubuntu/Linux)

This section is for users who want a **double-click app** with minimal terminal use.

### Option A: AppImage (Recommended)
1. Ask a technical teammate to provide `NexusCodex.AppImage` (one file) from this repo, or
   download it from your team’s release channel.
2. Save the file in your **Downloads** folder.
3. Right‑click the file → **Properties** → **Permissions** → check **Allow executing file as program**.
4. Double‑click **NexusCodex.AppImage** to launch the desktop app.

### Option B: Prebuilt Folder (AppDir Tarball)
If you receive a file named `nexus-desktop.AppDir.tar.gz`:
1. Right‑click the file → **Extract Here**.
2. Open the new folder.
3. Double‑click **AppRun** to launch.

### If the app doesn’t open
- Try right‑click → **Run** instead of double‑click.
- If Ubuntu shows a security warning, choose **Trust & Launch**.
- If the window opens and closes immediately, ask a teammate to run it once from a terminal
  to capture the error message.

### Where data is stored
- The app stores settings and data in your **user config directory** (Linux: `~/.config/nexus`).

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
