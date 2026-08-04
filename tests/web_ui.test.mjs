import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const webApp = readFileSync("apps/web/src/main.tsx", "utf8");
const webCss = readFileSync("apps/web/src/styles.css", "utf8");

describe("web UI", () => {
  it("renders table names with real newlines, not escaped newline text", () => {
    assert.match(webApp, /join\("\\n"\)/);
    assert.doesNotMatch(webApp, /join\("\\\\n"\)/);
  });

  it("keeps the tables list inside the source panel", () => {
    assert.match(webCss, /\.tables-list \{/);
    assert.match(webCss, /overflow: auto/);
    assert.match(webCss, /min-height: 0/);
    assert.doesNotMatch(webCss, /\.tables-list \{[^}]*display: flex/);
    assert.doesNotMatch(webCss, /\.tables-list \{[^}]*justify-content: center/);
  });
});
