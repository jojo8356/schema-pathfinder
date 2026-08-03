import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

describe("contract fixtures", () => {
  it("define declared FK metadata without business rows", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");

    assert.equal(Array.isArray(fixture.edges), true);
    assert.equal(fixture.edges.length, 3);

    for (const edge of fixture.edges) {
      assert.equal(edge.evidence.includes("declared_fk"), true);
      assert.equal(Object.hasOwn(edge, "rows"), false);
      assert.equal(Object.hasOwn(edge, "sampleData"), false);
    }
  });

  it("locks expected path, score, evidence, and renderer outputs", () => {
    const expected = readJson("fixtures/postgres/dressshot_expected_clothing_item_user.json");
    const [path] = expected.paths;

    assert.equal(path.score, 156);
    assert.equal(path.length, 3);
    assert.deepEqual(path.evidence, ["declared_fk"]);
    assert.deepEqual(path.tables, [
      "ClothingItem",
      "ClothingSession",
      "SellerProfile",
      "User"
    ]);
    assert.match(path.renderers.equation, /ClothingItem\.clothingSessionId = ClothingSession\.id/);
    assert.match(path.renderers.sql, /limit 50;/);
    assert.match(path.renderers.text, /Path 1 score 156/);
    assert.match(path.renderers.mermaid, /flowchart LR/);
  });

  it("locks no-path semantics as a normal result", () => {
    const expected = readJson("fixtures/postgres/dressshot_expected_no_path.json");

    assert.deepEqual(expected.paths, []);
    assert.equal(expected.noPathReason, "NO_DECLARED_FK_PATH");
  });
});
