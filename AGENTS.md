# Repository Guidance

- Files should be ~300 lines as a soft rule; split large source files when there is a clear boundary.
- Functions should fit on one screen; split large functions around behavior.
- Do not add clippy exceptions by default. Prefer changing code, visibility, tests, or module structure. If needed, keep the exception narrow and justify it in the commit, PR, or final handoff.
- Do not add comments in source code. Prefer clear names, smaller functions, and tests over inline explanations.
- Remove existing comments from touched source code when they are no longer needed. MPL-2.0 license headers (`// SPDX-License-Identifier: MPL-2.0`) are fine and should be kept.
- Do not edit the template `justfile` unless explicitly asked.
- Before committing, update `docs/spec.md` when behavior or user-facing expectations change.
- Before committing, run `just check` and `cargo test` and `cargo fmt`, then fix all warnings, errors, and failures.
- Do not add agent or AI attribution to commit messages (no `Co-Authored-By: Claude` or similar).

## Repository map

- `.github/` contains CI and release workflows.
- `docs/` contains the product specification and local development guidance.
- `fixtures/` contains provider API responses and probe scripts used during development.
- `i18n/` contains Fluent translations embedded into the binary.
- `packaging/` contains distribution manifests and vendored Cargo source metadata.
- `resources/` contains desktop metadata, provider icons, screenshots, and UI prototypes.
- `scripts/` contains screenshot and development utilities.
- `src/` contains the Rust application, shared runtime, UI, storage, and provider integrations.

Each maintained subdirectory has a local `AGENTS.md` with its file map. Read the most specific applicable file before editing there.

Issues are tracked as local markdown files under `issues/`.

This is a single-context repo. `docs/spec.md` is the current product/domain spec; ADRs may be added under `docs/adr/`.

Local RTK usage guidance may be available in the intentionally untracked `docs/RTK.md`.

Graphify-specific workflow guidance is in `docs/graphify.md`; read it only when the user invokes `/graphify`.
