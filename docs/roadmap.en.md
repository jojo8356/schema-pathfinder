# Roadmap

## Short term

- Add a wider variety of schema fixtures.

### Done (v0.1.4)

- **Stabilize the CLI commands around `tables`, `schemas`, `databases`, `tree`, and `path`.**
  Parsing now lives in a pure module (`schema_pathfinder::cli`) that can be tested without spawning
  the binary: stable subcommands with options accepted before or after, `dbs` as an alias,
  per-command help, `--version`, `0`/`1`/`2` exit codes, rejection of ambiguous invocations (two
  sources, two legacy flags, unknown command), and the legacy `--tables`/`--schemas`/`--tree` flags
  kept for compatibility.
- **Strengthen tests for the PostgreSQL SQL parser.**
  Dollar-quoted bodies, comments, `ALTER TABLE ... DROP CONSTRAINT`, composite keys, multiple
  schemas, three-part names, constraints without a name or a referenced column: 33 integration tests
  (`tests/sql_metadata.rs`) plus 12 white-box unit tests in the module, and a direct comparison
  between `dressshot_schema.sql` and the `dressshot_seed_fk_edges.json` fixture.
- **Improve the desktop layout on small and large screens.**
  One fluid layout: the Path panel puts one control per line, the Source column is bounded
  (240 / 340 / 440 px) and gives up space first, the result keeps a 340 px floor, the workspace is
  centred and capped at 1800 px, minimum window 640 x 520. Zero Slint `binding loop`: the layout
  never reads the window geometry and uses no ternary expression.

## Medium term

- Export a Markdown or HTML report from the CLI.
- More explainable weighting of candidate paths.
- Schema filtering in the CLI and API.
- Richer Mermaid mode to visualize intermediate tables.
- Package installation docs for Debian and AppImage, with screenshots.

## Long term

- Optional heuristics to detect likely undeclared relationships.
- Connectors for other relational databases.
- Audit mode to flag isolated tables or missing relationships.
- A UI to compare two schema versions.

## Current non-goals

- Reading or profiling business data.
- Modifying the database schema.
- Replacing a full modeling tool.
- Building a multi-tenant SaaS platform before the local product has stabilized.
