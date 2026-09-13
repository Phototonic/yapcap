# Legacy Login Helpers

- `mod.rs` exposes legacy login helpers.
- `code_entry.rs` handles Claude authorization-code entry and Copilot device-code copying.
- `cursor.rs` handles Cursor's legacy scan/login path.
- `metadata.rs` synchronizes Codex and Claude account metadata after refresh and clears legacy display snapshots.

Prefer the current flow modules for new providers or behavior.
