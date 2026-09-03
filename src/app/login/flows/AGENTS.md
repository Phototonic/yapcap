# Login Flows

- `mod.rs` defines flow-level shared types and dispatch support.
- `antigravity.rs`, `claude.rs`, `codex.rs`, `copilot.rs`, and `gemini.rs` implement browser/device OAuth flows.
- `kimi.rs`, `minimax.rs`, and `opencode_go.rs` implement API-key account flows.

Provider-specific network and account-storage details belong in `src/providers/`; these modules adapt them to UI login state.
