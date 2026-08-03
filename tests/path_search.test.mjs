import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { findPaths } from "../packages/core/src/path_search.mjs";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

describe("path search", () => {
  it("returns ordered paths with FK edges in traversal order", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const result = findPaths({
      edges: fixture.edges,
      sourceTable: "ClothingItem",
      targetTable: "User"
    });

    assert.equal(result.paths.length, 1);
    assert.equal(result.paths[0].length, 3);
    assert.deepEqual(
      result.paths[0].edges.map((edge) => edge.constraintName),
      [
        "ClothingItem_clothingSessionId_fkey",
        "ClothingSession_sellerProfileId_fkey",
        "SellerProfile_userId_fkey"
      ]
    );
  });

  it("returns no-path result for disconnected targets", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const result = findPaths({
      edges: fixture.edges,
      sourceTable: "User",
      targetTable: "ClothingItem"
    });

    assert.deepEqual(result.paths, []);
    assert.equal(result.noPathReason, "NO_DECLARED_FK_PATH");
  });

  it("returns no-path result when max depth is exhausted", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const result = findPaths({
      edges: fixture.edges,
      sourceTable: "ClothingItem",
      targetTable: "User",
      maxDepth: 2
    });

    assert.deepEqual(result.paths, []);
    assert.equal(result.noPathReason, "NO_DECLARED_FK_PATH");
  });

  it("resolves capitalized Prisma-style table names", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const result = findPaths({
      edges: fixture.edges,
      sourceTable: "ClothingItem",
      targetTable: "SellerProfile"
    });

    assert.equal(result.paths[0].edges[0].from.table, "ClothingItem");
    assert.equal(result.paths[0].edges.at(-1).to.table, "SellerProfile");
  });
});
