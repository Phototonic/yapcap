# 05: Migrate OpenCode Go and Contract Duplicate Key Authentication

**Parent:** Deepen Provider Account Architecture

**What to build:** Migrate OpenCode Go to the shared key-authentication lifecycle, verify all three provider adapters, and remove obsolete parallel lifecycle implementation that no caller needs afterward.

**Blocked by:** 03: Establish Shared Key Authentication With Kimi; 04: Migrate Minimax to Shared Key Authentication.

**Status:** completed

**Type:** AFK

- [x] OpenCode Go uses the shared editing, masking, visibility, provenance, validation, and save lifecycle.
- [x] OpenCode credentials prefill the form when available and typed input clears imported-key provenance.
- [x] Successful add and reauthentication preserve OpenCode Go account identity rules, select the account, clear login state, and request refresh.
- [x] OpenCode Go-specific key discovery, legacy environment fallback, active-account matching, account construction, persistence, and language remain in its adapter.
- [x] Kimi, Minimax, and OpenCode Go expose consistent lifecycle behavior through three concrete adapters at the seam.
- [x] Successful saves continue to clear the form immediately without an observable confirmation state.
- [x] Dormant success states and provider-specific lifecycle implementation made obsolete by all three migrations are removed.
- [x] Common behavior tests cover all three adapters, with focused tests retaining their real differences.
- [x] No OAuth, device authorization, pasted-code, or Cursor scan behavior is routed through the key-authentication module.
- [x] `just check`, `cargo test`, and `cargo fmt` pass without new lint exceptions.
