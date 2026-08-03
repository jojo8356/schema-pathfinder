# 0001 - Architecture Spine

`schema-pathfinder` follows the BMAD architecture spine created for the standalone pathfinder project.

## Decisions

- AD-1: Repository independence.
- AD-2: Framework-independent TypeScript core.
- AD-3: Direct CLI, not NestJS CLI.
- AD-4: NestJS is an adapter, not the engine owner.
- AD-5: Desktop UI uses the CLI locally and NestJS remotely.
- AD-6: One output contract.
- AD-7: Schema metadata only.
- AD-8: UI stack split.
- AD-9: Admin/local exposure only.
- AD-10: Release channels are independent but contract-bound.

## Consequences

- DressShot is a validation target, not the owning application.
- Pathfinding logic belongs in `packages/core`.
- CLI, API, web, and desktop must consume the shared contract.
- UI surfaces may display renderer output but must not rebuild path strings independently.
- PostgreSQL application rows are never inspected for MVP path discovery.
- Generated SQL is text output only and is not executed by UI surfaces.
