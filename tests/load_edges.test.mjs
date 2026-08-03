import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { loadEdges } from "@schema-pathfinder/cli/load_edges";

describe("CLI edge loading", () => {
  it("loads FK metadata from explicit fixture", async () => {
    const edges = await loadEdges({
      fixture: "fixtures/postgres/dressshot_seed_fk_edges.json"
    });

    assert.equal(edges.length, 3);
    assert.equal(edges[0].evidence.includes("declared_fk"), true);
  });

  it("fails without DATABASE_URL or fixture without leaking secrets", async () => {
    await assert.rejects(
      () => loadEdges({}, {}),
      (error) => {
        assert.equal(error.code, "DB_CONFIG_MISSING");
        assert.equal(error.exitCode, 1);
        assert.doesNotMatch(error.message, /postgres:\/\//);
        return true;
      }
    );
  });
});
