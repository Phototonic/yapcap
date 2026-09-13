# Account Settings Components

- `empty.rs` renders the no-accounts state.
- `login_controls.rs` renders add, login, import, and reauthentication controls.
- `rows.rs` renders account rows and account actions.

Account persistence and login execution stay behind the app/provider action layers. Grok import-availability checks currently read host credentials from `empty.rs` and the parent `accounts.rs`; account-row actions use registry-provided facts.
