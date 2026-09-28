import { existsSync } from "node:fs";

const requiredPaths = [
  "packages/core/src/index.ts",
  "packages/contracts/src/index.ts",
  "apps/cli-rust/src/main.rs",
  "apps/cli-rust/src/bin/schema-pathfinder-api.rs",
  "apps/web/src/main.tsx",
  "apps/desktop/src/main.rs",
  "fixtures/postgres/dressshot_seed_fk_edges.json",
  "docs/decisions/0001-architecture-spine.md"
];

const missingPaths = requiredPaths.filter((path) => !existsSync(path));

if (missingPaths.length > 0) {
  console.error(`Missing scaffold paths: ${missingPaths.join(", ")}`);
  process.exit(1);
}

console.log("schema-pathfinder scaffold ok");
