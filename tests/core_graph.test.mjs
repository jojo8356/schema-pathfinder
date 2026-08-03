import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  createSchemaGraph,
  outgoingEdges,
  resolveTable,
  tableKey
} from "../packages/core/src/graph.mjs";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

describe("core graph", () => {
  it("builds an in-memory graph from declared FK edges", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const graph = createSchemaGraph(fixture.edges);

    assert.equal(graph.edges.length, 3);
    assert.equal(graph.nodes.size, 4);
    assert.equal(graph.graph.isDirected(), true);
    assert.equal(graph.graph.isMultigraph(), true);
    assert.equal(outgoingEdges(graph, { schema: "public", table: "ClothingItem" }).length, 1);
    assert.equal(outgoingEdges(graph, { schema: "public", table: "User" }).length, 0);
  });

  it("resolves Prisma-style table names with capitals", () => {
    const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
    const graph = createSchemaGraph(fixture.edges);

    assert.deepEqual(resolveTable(graph, "ClothingItem"), {
      schema: "public",
      table: "ClothingItem"
    });
  });

  it("creates stable table keys", () => {
    assert.equal(tableKey({ schema: "public", table: "User" }), "public.User");
  });
});
