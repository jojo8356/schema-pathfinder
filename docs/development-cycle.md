# Development Cycle

The implementation order follows the BMAD readiness correction.

## Order

1. Repository scaffold and docs.
2. Shared fixtures and contract tests.
3. Core contracts and in-memory graph.
4. Metadata-only PostgreSQL FK discovery.
5. Path search and no-path result.
6. Scoring and shared renderers.
7. CLI.
8. Rust admin/local API binary.
9. Web UI consuming the Rust API contract.
10. Desktop UI using the Rust core locally, with no backend URL.
11. DressShot validation and release hardening.

## Definition of done

- Contract tests pass before surface tests.
- No committed secrets.
- No application rows are read.
- Shared output contract is preserved.
