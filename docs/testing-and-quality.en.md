# Testing and Quality

## Main commands

```bash
pnpm test                    # 76 Node tests
pnpm typecheck
pnpm lint
cargo test --workspace       # 152 Rust tests (unit, integration, doctests)
cargo build --release -p schema-pathfinder --bin schema-pathfinder
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## What is tested

- JSON fixture contracts.
- Path rendering in text, equation, SQL, Mermaid, and JSON formats.
- PostgreSQL SQL parsing into metadata: 33 integration tests
  (`apps/cli-rust/tests/sql_metadata.rs`) and 12 white-box unit tests in
  `postgres_sql_metadata.rs` (statement splitting, comments, dollar-quoted bodies,
  composite keys, `DROP CONSTRAINT`).
- The CLI surface: pure parsing in `src/cli.rs` (23 unit tests) and 24 end-to-end
  tests that run the compiled binary through `CARGO_BIN_EXE_schema-pathfinder` and
  check stdout, stderr and the exit code (`0`, `1`, `2`).
- The desktop UI contract (`tests/desktop_ui.test.mjs`): responsive structure, no
  pinned window size, and no horizontal size, padding or `font-size` bound to
  `root.width` (Slint binding loop).
- Expected monorepo scaffolding.
- Rust behaviors of the pathfinder core.

## CI quality gates

The `Node and Rust tests` job of `.github/workflows/build-binaries.yml` runs, in order: the
desktop UI contract, the Node tests, `pnpm lint`, `cargo test --workspace`, a Slint `binding loop`
check on the desktop UI, then the API binary build. The packaging job (`.deb`, AppImage) depends on
it: no package is produced when the tests fail.

## Quality strategy

The project favors tests close to the user contracts: source input, expected path, expected rendering. This approach protects the CLI, web, and desktop surfaces against format drift.

## Useful manual verification

For a real PostgreSQL database:

```bash
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

For a local fixture:

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```

## Pre-release criteria

- All tests pass.
- Release binaries compile.
- The docs contain no real secrets.
- Generated artifacts do not pollute the commit unless publishing is intentional.
- The README stays sufficient to evaluate the project in under a few minutes.
