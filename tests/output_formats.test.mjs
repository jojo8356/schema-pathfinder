import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { formatList, outputFormats, parseOutputFormat } from "@schema-pathfinder/core/output_formats";
import { renderPath } from "@schema-pathfinder/core/renderers";

describe("output format registry", () => {
  it("declares supported formats once", () => {
    assert.deepEqual(outputFormats, ["text", "equation", "json", "sql", "mermaid"]);
    assert.equal(formatList(), "text, equation, json, sql, mermaid");
  });

  it("validates formats through zod", () => {
    assert.equal(parseOutputFormat("sql").success, true);
    assert.equal(parseOutputFormat("xml").success, false);
  });

  it("rejects unsupported renderer formats with supported list", () => {
    const path = {
      source: { schema: "public", table: "A" },
      target: { schema: "public", table: "B" },
      score: 0,
      length: 0,
      evidence: [],
      edges: []
    };

    assert.throws(
      () => renderPath(path, "xml"),
      /Unsupported output format: xml. Supported formats: text, equation, json, sql, mermaid/
    );
  });
});
