# Contributing

## Set up the environment

```bash
pnpm install
cargo test -p schema-pathfinder
pnpm test
```

## Expected style

- Keep the pathfinding core independent from the UI surfaces.
- Prefer errors with stable codes.
- Never commit secrets, local dumps, or real PostgreSQL URLs.
- Add or update a fixture whenever a parsing or rendering behavior changes.
- Document new CLI commands in `docs/cli-reference.en.md`.

## Before committing

```bash
pnpm test
pnpm typecheck
cargo test -p schema-pathfinder
```

For packaging changes:

```bash
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
node scripts/build_desktop_deb.mjs
node scripts/build_desktop_appimage.mjs
```

## Commit messages

Recommended format:

```text
feat: add postgres architecture tree command
fix: keep desktop layout scrollable
docs: add employer repository documentation
```

## Review

A pull request should explain:

- the problem it solves;
- the surfaces it changes;
- the test commands that were run;
- any known limitations.
