# Nexus Codex Release Pipeline

## Build Steps
1. Run `cargo test` locally and in sandbox mode.
2. Run `cargo run -- bench cache --root .` and `cargo run -- bench vector --docs 500`.
3. Update the audit report via `cargo run -- audit mark --performance --security --docs`.

## Packaging
- Build cross-platform binaries using `cargo build --release`.
- Bundle the desktop app using platform-specific packaging tools.

## Verification
- Verify the kill switch and sandbox operations.
- Confirm dashboard availability at `127.0.0.1:8888`.

## Distribution
- Upload binaries to the release registry.
- Announce releases to beta testers before public rollout.
