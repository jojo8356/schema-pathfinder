# Testing and Quality

## Main commands

```bash
pnpm test
pnpm typecheck
pnpm lint
cargo test -p schema-pathfinder
cargo build --release -p schema-pathfinder --bin schema-pathfinder
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## What is tested

- JSON fixture contracts.
- Path rendering in text, equation, SQL, Mermaid, and JSON formats.
- PostgreSQL SQL parsing into metadata.
- Main CLI commands.
- Expected monorepo scaffolding.
- Rust behaviors of the pathfinder core.

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
