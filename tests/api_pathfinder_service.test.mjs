import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPathfinderService } from "../apps/api/src/pathfinder/pathfinder_service.mjs";

function fixtureLoader() {
  return async () => {
    const fixture = JSON.parse(readFileSync("fixtures/postgres/dressshot_seed_fk_edges.json", "utf8"));
    return fixture.edges;
  };
}

describe("NestJS pathfinder service adapter", () => {
  it("delegates valid path requests to the core", async () => {
    const service = createPathfinderService(fixtureLoader());
    const result = await service.findPath({
      sourceTable: "ClothingItem",
      targetTable: "User",
      format: "equation"
    });

    assert.equal(result.paths.length, 1);
    assert.match(result.rendered, /ClothingItem\.clothingSessionId = ClothingSession\.id/);
  });

  it("returns stable error code and title for invalid format", async () => {
    const service = createPathfinderService(fixtureLoader());
    const result = await service.findPath({
      sourceTable: "ClothingItem",
      targetTable: "User",
      format: "xml"
    });

    assert.deepEqual(result.error, {
      code: "UNSUPPORTED_FORMAT",
      title: "Unsupported output format"
    });
  });
});
