import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const desktopUi = readFileSync("apps/desktop/ui/main.slint", "utf8");
const desktopMain = readFileSync("apps/desktop/src/main.rs", "utf8");

function segmentBetween(source, startNeedle, endNeedle) {
  const start = source.indexOf(startNeedle);
  assert.notEqual(start, -1, startNeedle + " should exist");

  const end = source.indexOf(endNeedle, start);
  assert.notEqual(end, -1, endNeedle + " should exist after " + startNeedle);

  return source.slice(start, end);
}

function windowSection() {
  return segmentBetween(desktopUi, "export component AppWindow inherits Window {", "VerticalLayout {");
}

function countMatches(source, pattern) {
  const matches = source.match(pattern);
  if (matches === null) {
    return 0;
  }
  return matches.length;
}

describe("desktop UI", () => {
  it("does not render fake scrollbar components", () => {
    assert.doesNotMatch(desktopUi, /component TableScrollbar inherits Rectangle/);
    assert.doesNotMatch(desktopUi, /TableScrollbar \{ \}/);
    assert.doesNotMatch(desktopUi, /component GlobalScrollbar inherits Rectangle/);
    assert.doesNotMatch(desktopUi, /GlobalScrollbar \{ \}/);
  });

  it("uses one load button with a source kind selector", () => {
    const sourceSection = segmentBetween(desktopUi, 'text: "Source";', 'text: "Tables";');

    assert.equal(sourceSection.includes('model: ["fixture", "sql", "postgres"];'), true);
    assert.equal(sourceSection.includes("selected-source-kind"), true);
    assert.equal(sourceSection.includes("text: root.source-input-label;"), true);
    assert.equal(sourceSection.includes("text <=> root.source-input-value;"), true);
    assert.equal(sourceSection.includes("root.select-source(value, root.selected-source-kind, root.source-input-value);"), true);
    assert.equal(sourceSection.includes("root.load-source(root.selected-source-kind, root.source-input-value);"), true);
    assert.match(sourceSection, /text: "Load"/);
    assert.doesNotMatch(sourceSection, /text: "Load file"/);
    assert.doesNotMatch(sourceSection, /text: "Load SQL"/);
    assert.doesNotMatch(sourceSection, /text: "Load Postgres"/);
    assert.doesNotMatch(sourceSection, /text: "Fixture file"/);
    assert.doesNotMatch(sourceSection, /text: "SQL file"/);
    assert.doesNotMatch(sourceSection, /text: "Postgres URL"/);
    assert.equal(countMatches(sourceSection, /LineEdit \{/g), 1);
  });

  it("shows a source databases box", () => {
    const databasesSection = segmentBetween(desktopUi, 'text: "Databases";', 'text: "Tables";');

    assert.equal(desktopUi.includes("databases-text"), true);
    assert.match(databasesSection, /text: "database"/);
    assert.match(databasesSection, /text: root.databases-text/);
    assert.match(databasesSection, /MonospaceList \{/);
    assert.match(desktopUi, /component MonospaceList inherits ScrollView \{/);
  });

  it("uses selectors for schema, source table, and target table", () => {
    const pathSection = segmentBetween(desktopUi, 'text: "Path";', 'text: "Format";');

    assert.equal(pathSection.includes('text: "Schema";'), true);
    assert.equal(pathSection.includes('text: "From table";'), true);
    assert.equal(pathSection.includes('text: "To table";'), true);
    assert.equal(pathSection.includes("model: root.schema-options;"), true);
    assert.equal(pathSection.includes("model: root.table-options;"), true);
    assert.equal(pathSection.includes("root.select-schema(value);"), true);
    assert.equal(pathSection.includes("root.source-table = value;"), true);
    assert.equal(pathSection.includes("root.target-table = value;"), true);
    assert.doesNotMatch(pathSection, /LineEdit/);
  });

  it("keeps list scrollbars hidden until content overflows", () => {
    const listSection = segmentBetween(desktopUi, "component MonospaceList inherits ScrollView {", "component SourcePanel");

    assert.match(listSection, /vertical-scrollbar-policy: ScrollBarPolicy\.as-needed/);
    assert.match(listSection, /horizontal-scrollbar-policy: ScrollBarPolicy\.always-off/);
    assert.doesNotMatch(desktopUi, /ScrollBarPolicy\.always-on/);
  });

  it("keeps the window resizable instead of pinning a fixed size", () => {
    const windowProperties = windowSection();

    assert.doesNotMatch(windowProperties, /^\s*width: \d/m);
    assert.doesNotMatch(windowProperties, /^\s*height: \d/m);
    assert.match(windowProperties, /preferred-width: \d+px;/);
    assert.match(windowProperties, /preferred-height: \d+px;/);
    assert.match(windowProperties, /min-width: \d+px;/);
    assert.match(windowProperties, /min-height: \d+px;/);
    assert.doesNotMatch(desktopMain, /set_maximized/);
  });

  it("adapts the layout to the window width", () => {
    assert.match(desktopUi, /property <bool> compact: root\.width < \d+px;/);
    assert.match(desktopUi, /if !root\.compact: HorizontalLayout \{/);
    assert.match(desktopUi, /if root\.compact: ScrollView \{/);
    assert.match(desktopUi, /viewport-height: Math\.max\(self\.visible-height, \d+px\);/);
  });

  it("lets the panels stretch instead of hard coding panel sizes", () => {
    const layoutSection = segmentBetween(desktopUi, "export component AppWindow inherits Window {", "\n}\n");

    assert.doesNotMatch(layoutSection, /preferred-height: 700px/);
    assert.doesNotMatch(layoutSection, /Panel \{\n\s+width: \d+px;/);
    assert.match(desktopUi, /component SourcePanel inherits Panel \{/);
    assert.match(desktopUi, /component PathPanel inherits Panel \{/);
    assert.match(desktopUi, /component ResultPanel inherits Panel \{/);
    assert.match(layoutSection, /vertical-stretch: 1;/);
  });

  it("keeps the header minimal and removes the footer", () => {
    assert.equal(desktopUi.includes('text: "Schema Pathfinder";'), true);
    assert.doesNotMatch(desktopUi, /Local FK path inspector/);
    assert.doesNotMatch(desktopUi, /Fullscreen/);
    assert.doesNotMatch(desktopUi, /toggle-fullscreen/);
    assert.doesNotMatch(desktopUi, /text: root.status/);
    assert.doesNotMatch(desktopUi, /height: 34px/);
  });
});
