# Minimax Provider

- `mod.rs` exposes Minimax provider operations.
- `account.rs` manages labeled API-key accounts.
- `login.rs` implements API-key login and reauthentication.
- `opencode.rs` supports optional one-time OpenCode key prefill; environment-source Active matching lives in `src/providers/adapters.rs`.
- `storage.rs` reads and writes private API-key files; account metadata lives in COSMIC configuration.
