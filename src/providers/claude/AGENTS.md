# Claude Provider

- `mod.rs` exposes Claude provider operations.
- `account.rs` lists and manages YapCap-owned Claude accounts.
- `account/` contains account-specific helpers and tests.
- `host_session.rs` matches the active host Claude account from local state.
- `limits.rs` models Claude usage limits and windows.
- `login.rs` implements native browser OAuth login.
- `oauth.rs` fetches usage and refreshes OAuth tokens against Anthropic.
