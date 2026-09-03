# Application

- `mod.rs` owns `AppModel`, messages, application wiring, and top-level state transitions.
- `applet.rs` renders the panel applet.
- `popup_view.rs` owns the popup shell and shared widgets.
- `provider_actions.rs` handles provider/account actions from the UI.
- `provider_assets.rs` loads provider artwork.
- `refresh.rs` coordinates refresh messages and results.
- `session.rs` manages popup/session lifecycle state.
- `state.rs` defines app-local UI state.
- `tests.rs` contains app-level tests.
- `window.rs` handles popup window behavior.
- `host_auth_watch.rs` watches host CLI auth files on Linux.
- `login/` contains login flow implementations and tests.
- `popup_view/` contains popup detail, badges, and settings views.
