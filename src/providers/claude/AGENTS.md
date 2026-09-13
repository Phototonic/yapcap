# Claude Provider

- `mod.rs` fetches and normalizes Claude usage, coordinating token refresh and metadata updates.
- `account.rs` lists and manages YapCap-owned Claude accounts.
- `account/` contains account-specific tests.
- `host_session.rs` matches the active host Claude account from local state.
- `limits.rs` models Claude usage limits and windows.
- `login.rs` implements native browser OAuth login.
- `oauth.rs` parses OAuth token responses and refreshes tokens against Anthropic.
