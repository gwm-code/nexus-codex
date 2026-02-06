# Nexus Codex API Reference

## CLI Commands

### Cache
- `cache-warm`: Build cache metadata for a repository.
- `cache-diff`: Show file changes since the last cache snapshot.
- `cache-handshake`: Produce a handshake payload that captures repository metadata.
- `cache-payload`: Emit a diff payload with updated file contents.

### Memory
- `memory set <key> <value>`: Save a preference.
- `memory get <key>`: Retrieve a preference.
- `memory list`: List preferences.

### Sandbox
- `sandbox`: Shadow-run a command inside Docker.
- `sandbox-test`: Execute a test command inside Docker.

### Swarm
- `swarm plan`: Decompose tasks into a dependency-aware plan.
- `swarm run`: Execute swarm tasks in parallel.
- `swarm merge`: Merge a branch and report conflicts.

### Interface
- `serve`: Launch the dashboard.
- `daemon`: Run the background interface daemon.

### Vector
- `vector add`: Store a document in the vector store.
- `vector query`: Retrieve nearest documents.

### Audit
- `audit report`: View the audit checklist.
- `audit scan`: Run a security scan.
- `audit mark`: Mark audit items complete.
