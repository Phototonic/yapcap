# Rust Source

`main.rs` starts the COSMIC applet. Top-level modules provide application state, configuration, persistence, authentication, refresh orchestration, display formatting, and provider integrations.

- `account_selection.rs` handles selected-account preferences.
- `auth.rs` parses shared authentication claims.
- `config.rs` defines COSMIC settings entries.
- `currency_format.rs` formats credit and currency values.
- `debug_env.rs` provides debug-only environment behavior.
- `demo_env.rs` provides demo data and debug presentation support.
- `detection.rs` detects locally available provider sources.
- `error.rs` defines application error types and user-facing error conversion.
- `i18n.rs` initializes embedded translations.
- `key_authentication.rs` handles shared API-key account behavior.
- `logging.rs` initializes file logging.
- `model.rs` defines shared usage and application state types.
- `provider_enablement.rs` manages provider enablement preferences.
- `refresh_owner.rs` coordinates refresh ownership between processes.
- `runtime.rs` loads, refreshes, and persists shared runtime state.
- `shared_state.rs` defines versioned shared COSMIC runtime/control state.
- `test_support.rs` contains test-only helpers.
- `updates.rs` checks GitHub releases.
- `usage_display.rs` formats usage values and reset windows.
- `account_storage/` provides secure per-account persistence.
- `app/` contains the COSMIC application, popup, login flows, and UI tests.
- `config/` contains configuration watchers.
- `providers/` contains the provider interface, registry, adapters, and implementations.
