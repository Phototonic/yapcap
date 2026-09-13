# Configuration

- `watch_update.rs` applies changed COSMIC configuration keys to the in-memory configuration. The application subscribes to configuration changes in `src/app/mod.rs`.

The parent `src/config.rs` defines the configuration entries and shared app ID; this directory contains its supporting modules.
