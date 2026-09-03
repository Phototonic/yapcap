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

Issues are tracked as local markdown files under `issues/`.

This is a single-context repo. `docs/spec.md` is the current product/domain spec; ADRs may be added under `docs/adr/`.

@docs/RTK.md

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
