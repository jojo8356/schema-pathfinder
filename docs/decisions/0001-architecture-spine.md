# 0001 - Architecture Spine

`schema-pathfinder` follows the BMAD architecture spine created for the standalone pathfinder project.

## Decisions

- AD-1: Repository independence.
- AD-2: Rust core owns metadata discovery, pathfinding, scoring, and renderers.
- AD-3: Direct Rust CLI, not a Node CLI.
- AD-4: The Rust API is a separate server binary, not required by CLI or desktop builds.
- AD-5: Desktop UI calls the Rust core locally and never requires a backend URL.
- AD-6: One output contract.
- AD-7: Schema metadata only.
- AD-8: UI stack split.
- AD-9: Admin/local exposure only.
- AD-10: Release channels are independent but contract-bound.

## Consequences

- DressShot is a validation target, not the owning application.
- Pathfinding logic belongs in `apps/cli-rust/src/pathfinder_core.rs`; TypeScript packages must stay contract-bound or be removed during migration.
- CLI, API, web, and desktop must consume the shared contract.
- UI surfaces may display renderer output but must not rebuild path strings independently.
- PostgreSQL application rows are never inspected for MVP path discovery.
- Generated SQL is text output only and is not executed by UI surfaces.
