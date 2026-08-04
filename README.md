# schema-pathfinder

Standalone PostgreSQL schema pathfinder.

## Goal

Answer one focused question:

```text
How does table A join table B?
```

The MVP uses declared PostgreSQL foreign keys from schema metadata only. It does not read application table rows.

## CLI-first workflow

The first usable surface is the CLI:

```bash
schema-pathfinder path ClothingItem User --format equation
```

The packaged CLI is a standalone Rust binary with no JavaScript runtime and no backend URL.

Useful local commands:

```bash
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path ClothingItem User --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```

Without `--fixture`, the CLI reads PostgreSQL metadata directly through `DATABASE_URL`.

## Surfaces

- `apps/cli-rust`: Rust CLI and Rust local/admin API binary.
- `packages/core`: TypeScript contract mirror retained for current JS contract tests during web migration.
- `apps/web`: React web UI.
- `apps/desktop`: Rust/Slint desktop UI.

## Safety

- Metadata-only schema inspection.
- No committed database secrets.
- Generated SQL is read-only text for inspection and is not executed by UI surfaces.
- Public hosted exposure is out of scope for the MVP.

## Output formats

- `text`: readable path summary with score and edges.
- `equation`: join equations with each `->` on a new line.
- `json`: stable contract output for every surface.
- `sql`: read-only JOIN query text with quoted identifiers and `LIMIT`.
- `mermaid`: path diagram text.

## Repository layout

```text
schema-pathfinder/
  packages/
    contracts/
    core/
  apps/
    cli-rust/
    web/
    desktop/
  fixtures/
    postgres/
  docs/
    decisions/
```

## First commands

```bash
pnpm install
pnpm test
pnpm typecheck
cargo test -p schema-pathfinder
cargo build --release -p schema-pathfinder --bin schema-pathfinder
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
```

The current scaffold tests use Node's built-in test runner and do not require dependency installation.

## Packaged Artifacts

The CLI artifacts are local-only and contain only the Rust CLI binary plus documentation and fixtures:

- `dist/schema-pathfinder_0.1.0_amd64.deb`
- `dist/schema-pathfinder-0.1.0-x86_64.AppImage`

The Rust API is built as `target/release/schema-pathfinder-api` and is a separate server binary.
