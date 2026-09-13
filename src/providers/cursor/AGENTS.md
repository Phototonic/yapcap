# Cursor Provider

- `mod.rs` exposes Cursor provider operations.
- `discovery.rs` lists valid YapCap-managed accounts from stored metadata and tokens; installation detection lives in `src/detection.rs`.
- `identity.rs` derives account identity from Cursor credentials.
- `maintenance.rs` handles cleanup and maintenance of managed Cursor state.
- `refresh.rs` fetches Cursor usage and refreshes managed data.
- `scan.rs` reads Cursor IDE local storage.
- `storage.rs` derives account IDs and managed directory paths; persistence uses `src/account_storage/` through scan, maintenance, and refresh operations.
- `types.rs` contains an unused legacy managed-account metadata schema and is not declared by `mod.rs`.
