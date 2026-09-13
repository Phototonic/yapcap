# Codex Provider

- `mod.rs` fetches and normalizes Codex usage, coordinating token refresh, unauthorized-request retries, and metadata updates.
- `account.rs` lists and manages YapCap-owned Codex accounts.
- `login.rs` implements the Codex browser OAuth login flow.
- `oauth.rs` handles Codex OAuth token operations.
- `refresh.rs` exchanges refresh tokens for renewed credentials and parses their expiration.
- `opencode_import.rs` imports compatible OpenCode OAuth credentials when explicitly requested.
- `tests.rs` tests Codex account and OAuth behavior.
