import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { findPathsByComplexity } from "@schema-pathfinder/core/path_search";
import { clampMaxLinks, rankPathsByComplexity } from "@schema-pathfinder/core/scoring";
import { renderPaths } from "@schema-pathfinder/core/renderers";

// Two independent join routes between A and D:
//   short  : A -> B -> D            (2 links)
//   longer : A -> C -> E -> D       (3 links)
// This mirrors the real case where the shortest route may cross a table that is
// not yet populated during data entry, so the user needs the longer options too.
function branchingEdges() {
  const table = (name) => ({ schema: "public", table: name });
  const edge = (constraintName, from, fromColumn, to, toColumn) => ({
    constraintName,
    from: table(from),
    fromColumn,
    to: table(to),
    toColumn,
    evidence: ["declared_fk"]
  });

  return [
    edge("A_bId_fkey", "A", "bId", "B", "id"),
    edge("B_dId_fkey", "B", "dId", "D", "id"),
    edge("A_cId_fkey", "A", "cId", "C", "id"),
    edge("C_eId_fkey", "C", "eId", "E", "id"),
    edge("E_dId_fkey", "E", "dId", "D", "id")
  ];
}

describe("paths by complexity", () => {
  it("orders every path by increasing number of links", () => {
    const result = findPathsByComplexity({
      edges: branchingEdges(),
      sourceTable: "A",
      targetTable: "D"
    });

    assert.equal(result.paths.length, 2);
    assert.deepEqual(
      result.paths.map((path) => path.length),
      [2, 3]
    );
    assert.deepEqual(
      result.paths[0].edges.map((path) => path.to.table),
      ["B", "D"]
    );
  });

  it("respects the max links limit", () => {
    const result = findPathsByComplexity({
      edges: branchingEdges(),
      sourceTable: "A",
      targetTable: "D",
      maxLinks: 2
    });

    assert.equal(result.paths.length, 1);
    assert.equal(result.paths[0].length, 2);
  });

  it("reports no-path when the source and target cannot be joined", () => {
    const result = findPathsByComplexity({
      edges: branchingEdges(),
      sourceTable: "D",
      targetTable: "A"
    });

    assert.deepEqual(result.paths, []);
    assert.equal(result.noPathReason, "NO_DECLARED_FK_PATH");
  });

  it("clamps the max links into the supported range", () => {
    assert.equal(clampMaxLinks(0), 1);
    assert.equal(clampMaxLinks(3), 3);
    assert.equal(clampMaxLinks(99), 8);
    assert.equal(clampMaxLinks(Number.NaN), 5);
  });

  it("ranks equal-length paths by score, shortest first", () => {
    const base = { source: { table: "A" }, target: { table: "D" }, evidence: ["declared_fk"] };
    const makePath = (length, score, constraintName) => ({
      ...base,
      length,
      score,
      edges: [{ constraintName }]
    });

    const ranked = rankPathsByComplexity([
      makePath(3, 200, "long-strong"),
      makePath(2, 10, "short-weak"),
      makePath(2, 90, "short-strong")
    ]);

    assert.deepEqual(
      ranked.map((path) => path.length),
      [2, 2, 3]
    );
    assert.deepEqual(
      ranked.map((path) => path.edges[0].constraintName),
      ["short-strong", "short-weak", "long-strong"]
    );
  });

  it("renders several numbered paths in one block", () => {
    const paths = findPathsByComplexity({
      edges: branchingEdges(),
      sourceTable: "A",
      targetTable: "D"
    }).paths;

    const text = renderPaths(paths, "text");
    assert.match(text, /Path 1 score/);
    assert.match(text, /Path 2 score/);

    const equation = renderPaths(paths, "equation");
    assert.match(equation, /Path 1 score \d+ length 2: A -> B -> D/);
    assert.match(equation, /Path 2 score \d+ length 3: A -> C -> E -> D/);

    const json = renderPaths(paths, "json");
    assert.equal(JSON.parse(json).length, 2);

    assert.equal(renderPaths([], "text"), "");
  });
});
