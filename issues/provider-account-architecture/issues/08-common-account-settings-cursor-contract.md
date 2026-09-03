# 08: Migrate Cursor and Contract Duplicate Account Settings

**Parent:** Deepen Provider Account Architecture

**What to build:** Bring Cursor's scan-based account behavior into the common account-settings module, then remove repeated provider-specific account-page construction that no caller needs.

**Blocked by:** 07: Migrate OAuth and Device Providers to Common Account Settings.

**Status:** completed

**Type:** AFK

- [x] Cursor uses the common implementation for selection, active state, empty state, rows, status, enablement, and actions.
- [x] Cursor's add action still starts local IDE scanning rather than browser or key authentication.
- [x] Cursor scan states retain idle, scanning, found, already-connected, failure, confirm, retry, and dismiss behavior.
- [x] Cursor reauthentication remains a rescan and retains its background status-refresh requirements.
- [x] Host-active Cursor identity remains accurate and is not overwritten by generic host-account reconciliation.
- [x] All nine providers now use one common account-settings construction path with provider-specific adapters only where behavior varies.
- [x] Repeated provider-specific account section and row-policy implementation is removed after all callers migrate.
- [x] Account-settings source modules are split at clear behavioral seams and move toward repository size guidance without pass-through shallow modules.
- [x] Tests cover Cursor-specific behavior and the common nine-provider account-settings path.
- [x] `just check`, `cargo test`, and `cargo fmt` pass without new lint exceptions.
