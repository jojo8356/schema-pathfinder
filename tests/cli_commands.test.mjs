import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const cli = readFileSync("apps/cli-rust/src/main.rs", "utf8");
const api = readFileSync("apps/cli-rust/src/bin/schema-pathfinder-api.rs", "utf8");
const core = readFileSync("apps/cli-rust/src/pathfinder_core.rs", "utf8");

describe("PostgreSQL server metadata commands", () => {
  it("adds CLI commands for databases and schemas", () => {
    assert.match(cli, /CommandMode::Databases/);
    assert.match(cli, /CommandMode::Schemas/);
    assert.match(cli, /values\[0\] == "databases"/);
    assert.match(cli, /values\[0\] == "schemas"/);
    assert.match(cli, /Long\("databases"\) \| Long\("dbs"\)/);
    assert.match(cli, /Long\("schemas"\)/);
  });

  it("queries pg_database and pg_namespace through Rust core", () => {
    assert.match(core, /POSTGRES_DATABASE_SQL/);
    assert.match(core, /from pg_database/);
    assert.match(core, /POSTGRES_SCHEMA_SQL/);
    assert.match(core, /from pg_namespace/);
    assert.match(core, /discover_postgres_databases/);
    assert.match(core, /discover_postgres_schemas/);
  });

  it("exposes API routes for databases and schemas", () => {
    assert.match(api, /\/api\/databases/);
    assert.match(api, /\/api\/schemas/);
    assert.match(api, /source_databases_response/);
    assert.match(api, /source_schemas_response/);
  });
});
