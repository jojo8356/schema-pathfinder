import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { findPaths } from "@schema-pathfinder/core/path_search";
import {
  renderEquation,
  renderJson,
  renderMermaid,
  renderPath,
  renderSql,
  renderText
} from "@schema-pathfinder/core/renderers";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function firstFixturePath() {
  const fixture = readJson("fixtures/postgres/dressshot_seed_fk_edges.json");
  const result = findPaths({
    edges: fixture.edges,
    sourceTable: "ClothingItem",
    targetTable: "User"
  });

  return result.paths[0];
}

describe("shared renderers", () => {
  it("renders text with score, length, table order, and edges", () => {
    const output = renderText(firstFixturePath());

    assert.match(output, /Path 1 score 156 declared_fk length 3/);
    assert.match(output, /ClothingItem -> ClothingSession -> SellerProfile -> User/);
    assert.match(output, /1\. ClothingItem\.clothingSessionId -> ClothingSession\.id/);
  });

  it("renders equation output chained by arrows", () => {
    assert.equal(
      renderEquation(firstFixturePath()),
      "ClothingItem.clothingSessionId = ClothingSession.id -> ClothingSession.sellerProfileId = SellerProfile.id -> SellerProfile.userId = User.id"
    );
  });

  it("renders read-only SQL text with quoted identifiers and limit", () => {
    const output = renderSql(firstFixturePath());

    assert.match(output, /^select \*/);
    assert.match(output, /from "ClothingItem" t0/);
    assert.match(output, /join "User" t3/);
    assert.match(output, /limit 50;$/);
  });

  it("renders JSON and Mermaid without UI-specific logic", () => {
    const path = firstFixturePath();
    const json = renderJson({ paths: [path] });
    const mermaid = renderMermaid(path);

    assert.equal(JSON.parse(json).paths[0].score, 156);
    assert.match(mermaid, /flowchart LR/);
    assert.match(mermaid, /SellerProfile --> User/);
  });

  it("routes supported formats through renderPath", () => {
    const path = firstFixturePath();

    assert.match(renderPath(path, "text"), /Path 1/);
    assert.match(renderPath(path, "equation"), / = /);
    assert.match(renderPath(path, "sql"), /select \*/);
    assert.match(renderPath(path, "mermaid"), /flowchart/);
    assert.match(renderPath(path, "json"), /"score": 156/);
  });
});
