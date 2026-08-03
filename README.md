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

The CLI imports the TypeScript core directly and must work without a NestJS server.

## Surfaces

- `packages/core`: TypeScript pathfinding engine, scoring, and renderers.
- `packages/cli`: direct Node CLI.
- `apps/api`: NestJS local/admin API adapter.
- `apps/web`: React web UI.
- `apps/desktop`: Rust/Slint desktop UI.

## Safety

- Metadata-only schema inspection.
- No committed database secrets.
- Generated SQL is read-only text for inspection and is not executed by UI surfaces.
- Public hosted exposure is out of scope for the MVP.

## Output formats

- `text`: readable path summary with score and edges.
- `equation`: join equations chained with `->`.
- `json`: stable contract output for every surface.
- `sql`: read-only JOIN query text with quoted identifiers and `LIMIT`.
- `mermaid`: path diagram text.

## Repository layout

```text
schema-pathfinder/
  packages/
    contracts/
    core/
    cli/
  apps/
    api/
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
```

The current scaffold tests use Node's built-in test runner and do not require dependency installation.
