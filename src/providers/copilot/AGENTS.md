# GitHub Copilot Provider

- `mod.rs` exposes Copilot provider operations.
- `account.rs` manages GitHub-identity-based accounts.
- `device_flow.rs` implements GitHub device login.
- `headers.rs` builds Copilot request headers.
- `login.rs` coordinates the Copilot login flow.
- `opencode_import.rs` imports compatible OpenCode credentials when explicitly requested.
- `parse.rs` parses free and paid Copilot usage schemas, including token-based credits and spend.
- `storage.rs` implements Copilot's account storage schema.
- `parse/` contains parser tests.
