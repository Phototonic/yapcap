# Application

- `mod.rs` owns `AppModel`, messages, application wiring, and top-level state transitions.
- `applet.rs` renders the panel applet.
- `popup_view.rs` owns the popup shell and shared widgets.
- `provider_actions.rs` handles provider/account actions from the UI.
- `provider_assets.rs` loads provider artwork.
- `refresh.rs` coordinates refresh messages and results.
- `session.rs` dispatches account deletion, login, imports, reauthentication, and metadata synchronization.
- `state.rs` implements shared `AppState` accessors, selection lookup, and runtime updates.
- `tests.rs` contains app-level tests.
- `window.rs` opens external URLs and schedules release checks with retry backoff.
- `host_auth_watch.rs` watches host authentication files and provider detection markers on Linux.
- `login/` contains login flow implementations and tests.
- `popup_view/` contains popup detail, badges, and settings views.
