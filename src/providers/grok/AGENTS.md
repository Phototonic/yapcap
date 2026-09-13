# Grok Provider

- `mod.rs` fetches billing data with managed tokens, refreshes credentials, and retries unauthorized requests.
- `account.rs` manages account identity, deduplication, host CLI credential reading, and host Active matching.
- `login.rs` implements browser sign-in, loopback callbacks, targeted reauthentication, CLI import, and account persistence.
- `oauth.rs` builds authorization URLs and PKCE values, exchanges and refreshes tokens, and decodes identity claims.
- `usage.rs` parses Grok billing and usage snapshots.
- `tests.rs` contains Grok provider tests. The OAuth loopback test supplies a fake browser launcher and local token/billing endpoints while exercising callback handling and managed account persistence.
