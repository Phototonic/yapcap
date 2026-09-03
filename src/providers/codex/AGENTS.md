# Codex Provider

- `mod.rs` exposes Codex provider operations.
- `account.rs` lists and manages YapCap-owned Codex accounts.
- `login.rs` implements the Codex browser OAuth login flow.
- `oauth.rs` handles Codex OAuth token operations.
- `refresh.rs` refreshes expired/unauthorized credentials and retries usage requests.
- `opencode_import.rs` imports compatible OpenCode OAuth credentials when explicitly requested.
- `tests.rs` tests Codex account and OAuth behavior.
