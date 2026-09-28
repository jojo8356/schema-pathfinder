import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";

describe("schema-pathfinder scaffold", () => {
  it("contains the standalone workspace structure", () => {
    const paths = [
      "apps/cli-rust",
      "packages/core",
      "packages/contracts",
      "apps/web",
      "apps/desktop",
      "fixtures/postgres",
      "docs/decisions"
    ];

    for (const path of paths) {
      assert.equal(existsSync(path), true, `${path} should exist`);
    }
  });

  it("keeps DressShot validation as fixture data, not project ownership", () => {
    const readme = readFileSync("README.md", "utf8");
    const fixture = readFileSync("fixtures/postgres/dressshot_seed_fk_edges.json", "utf8");

    assert.match(readme, /Standalone PostgreSQL schema pathfinder/);
    assert.match(fixture, /ClothingItem/);
  });
});
