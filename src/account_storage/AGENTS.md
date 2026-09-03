# Account Storage

- `mod.rs` implements secure per-provider account directories, metadata, tokens, snapshots, and JSON/file primitives.
- `tests.rs` tests directory creation, permissions, serialization, and account lifecycle behavior.

This is the shared persistence foundation. Preserve owner-only directory/file permissions and keep provider-specific schemas in provider modules unless a primitive is genuinely shared.
