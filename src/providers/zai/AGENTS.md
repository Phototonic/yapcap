# Z.AI Coding Plan Provider

- `mod.rs` fetches the fixed global quota endpoint using managed keys, disables redirects, and retries with raw authorization only after a bearer request returns 401.
- `account.rs` lists and applies labeled managed accounts.
- `login.rs` adapts shared API-key login and reauthentication with local label/key validation.
- `opencode.rs` discovers usable typed OpenCode keys for form prefill and content-aware provider detection.
- `quota.rs` parses five-hour, weekly, and MCP windows, including counter fallbacks and measured MCP durations.
- `storage.rs` normalizes, reads, writes, and deletes private API-key storage; account metadata lives in COSMIC configuration.

Z.AI has no environment-key fallback or host Active badge. Parser fixtures and their schema notes live under `fixtures/zai/`.
