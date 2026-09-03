# Graphify Workflow

Read this document only when the user invokes `/graphify`.

This project has a knowledge graph at `graphify-out/` with god nodes, community structure, and cross-file relationships.

- Use the installed graphify skill or these instructions before doing anything else.
- For codebase questions, first run `graphify query "<question>"` when `graphify-out/graph.json` exists.
- Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts.
- Dirty `graphify-out/` files are expected after hooks or incremental updates; do not skip graphify for that reason.
- Skip graphify only when the task concerns stale or incorrect graph output, or the user explicitly says not to use it.
- If `graphify-out/wiki/index.md` exists, use it for broad navigation instead of raw source browsing.
- Read `graphify-out/GRAPH_REPORT.md` only for broad architecture review or when query/path/explain do not provide enough context.
- After modifying code, run `graphify update .` to keep the graph current.
