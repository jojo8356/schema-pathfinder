# Employer Overview

## Summary

Schema Pathfinder is a standalone database-tooling project. It helps you quickly understand the relationships between PostgreSQL tables by leveraging their declared foreign keys. The project is built like a real technical product: CLI, desktop UI, web UI, API, contract fixtures, tests, packaging, and documentation.

## Problem addressed

On an existing database, understanding how two tables are connected can take time: you have to inspect schemas, constraints, intermediate tables, and sometimes several databases. Schema Pathfinder automates that search and returns a readable result in several formats.

## Technical value

- Reads PostgreSQL metadata without querying business data.
- Path search over a foreign-key graph.
- Rendering tailored to several use cases: human, SQL, diagram, JSON.
- Offline mode via a fixture or SQL file.
- Connected mode via a PostgreSQL URL.
- Distribution as binaries and installable packages.

## What you can observe in the repository

- A coherent multi-surface architecture: the Rust core is shared by the CLI, the API, and the desktop.
- A separate API for the web only, to keep the CLI and desktop self-contained.
- JSON fixtures to stabilize the contracts between surfaces.
- Automated tests on critical behaviors.
- Clear documentation for onboarding, auditing, and contributing.

## Deliberate scope

The MVP stays focused on declared foreign keys. It does not yet perform semantic inference on column names, statistical scoring on data, or reading of business rows. This choice makes the tool safer, more predictable, and easier to verify.

## How to evaluate the project quickly

```bash
pnpm test
cargo test -p schema-pathfinder
cargo run -p schema-pathfinder --bin schema-pathfinder -- --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

If no local database is available, the included fixture is enough:

```bash
cargo run -p schema-pathfinder --bin schema-pathfinder -- path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```
