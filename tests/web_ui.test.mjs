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

  it("shows databases in the source panel", () => {
    assert.match(webApp, /type DatabasesResponse/);
    assert.match(webApp, /\/api\/databases/);
    assert.match(webApp, /<h2>Databases<\/h2>/);
    assert.match(webApp, /renderNamesText\(databases, "NO_DATABASES"\)/);
    assert.match(webCss, /\.metadata-list/);
  });

  it("shows an explicit empty-table state", () => {
    assert.match(webApp, /function renderTablesText/);
    assert.match(webApp, /return "NO_TABLES"/);
    assert.match(webApp, /renderTablesText\(tables\)/);
  });

  it("keeps the tables list inside the source panel", () => {
    assert.match(webCss, /\.tables-list \{/);
    assert.match(webCss, /overflow: auto/);
    assert.match(webCss, /min-height: 0/);
    assert.doesNotMatch(webCss, /\.tables-list \{[^}]*display: flex/);
    assert.doesNotMatch(webCss, /\.tables-list \{[^}]*justify-content: center/);
  });
});
