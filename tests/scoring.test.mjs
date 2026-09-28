import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { findPaths } from "@schema-pathfinder/core/path_search";
import { rankPaths, scorePath } from "@schema-pathfinder/core/scoring";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

describe("deterministic scoring", () => {
  it("scores declared FK paths with hop penalties", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const result = findPaths({
      edges: fixture.edges,
      sourceTable: "ClothingItem",
      targetTable: "User"
    });

    assert.equal(result.paths[0].score, 156);
    assert.deepEqual(result.paths[0].evidence, ["declared_fk"]);
    assert.equal(result.paths[0].scoreContributions.at(-1).label, "extra_hop");
  });

  it("penalizes technical and auth/session tables deterministically", () => {
    const source = { schema: "public", table: "A" };
    const target = { schema: "public", table: "User" };
    const edges = [
      {
        constraintName: "A_sessionId_fkey",
        from: source,
        fromColumn: "sessionId",
        to: { schema: "public", table: "Session" },
        toColumn: "id",
        evidence: ["declared_fk"]
      },
      {
        constraintName: "Session_migrationId_fkey",
        from: { schema: "public", table: "Session" },
        fromColumn: "migrationId",
        to: { schema: "public", table: "_prisma_migrations" },
        toColumn: "id",
        evidence: ["declared_fk"]
      }
    ];

    const scored = scorePath(source, target, edges);

    assert.equal(scored.score, 64);
    assert.equal(
      scored.scoreContributions.some((item) => item.label === "technical_table"),
      true
    );
    assert.equal(
      scored.scoreContributions.some((item) => item.label === "auth_session_table"),
      true
    );
  });

  it("ranks paths by score and returns at most three", () => {
    const source = { schema: "public", table: "A" };
    const target = { schema: "public", table: "D" };
    const makePath = (score, constraintName) => {
      return {
        source,
        target,
        score,
        length: 1,
        evidence: ["declared_fk"],
        edges: [{ constraintName }]
      };
    };

    const ranked = rankPaths([
      makePath(10, "low"),
      makePath(40, "high"),
      makePath(30, "mid"),
      makePath(20, "fourth")
    ]);

    assert.deepEqual(
      ranked.map((path) => path.score),
      [40, 30, 20]
    );
  });
});
