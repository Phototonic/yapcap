# OpenCode Go Provider

- `mod.rs` exposes OpenCode Go provider operations.
- `account.rs` manages labeled API-key accounts and matches stored keys for the host Active badge.
- `login.rs` implements API-key login and reauthentication.
- `opencode.rs` discovers OpenCode keys for prefill and the Active lookup coordinated by `src/providers/adapters.rs`.
- `storage.rs` reads and writes private API-key files; account metadata lives in COSMIC configuration.
