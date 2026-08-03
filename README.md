# schema-pathfinder

Standalone PostgreSQL schema pathfinder.

## Goal

Answer one focused question:

```text
How does table A join table B?
```

The MVP uses declared PostgreSQL foreign keys from schema metadata only. It does not read application table rows.

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

## First commands

```bash
pnpm install
pnpm test
pnpm typecheck
```
