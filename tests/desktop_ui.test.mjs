import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const desktopUi = readFileSync("apps/desktop/ui/main.slint", "utf8");

function segmentBetween(source, startNeedle, endNeedle) {
  const start = source.indexOf(startNeedle);
  assert.notEqual(start, -1, startNeedle + " should exist");

  const end = source.indexOf(endNeedle, start);
  assert.notEqual(end, -1, endNeedle + " should exist after " + startNeedle);

  return source.slice(start, end);
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
    assert.match(databasesSection, /ScrollView \{/);
  });

  it("uses selectors for schema, source table, and target table", () => {
    const pathSection = segmentBetween(desktopUi, 'text: "Path";', 'text: "Format";');

    assert.equal(pathSection.includes('text: "Schema";'), true);
    assert.equal(pathSection.includes('text: "From table";'), true);
    assert.equal(pathSection.includes('text: "To table";'), true);
    assert.equal(pathSection.includes('model: root.schema-options;'), true);
    assert.equal(pathSection.includes('model: root.table-options;'), true);
    assert.equal(pathSection.includes('root.select-schema(value);'), true);
    assert.equal(pathSection.includes('root.source-table = value;'), true);
    assert.equal(pathSection.includes('root.target-table = value;'), true);
    assert.doesNotMatch(pathSection, /LineEdit/);
  });

  it("keeps the tables list scrollbar hidden until the table list overflows", () => {
    const tablesSection = segmentBetween(desktopUi, 'text: "Tables";', 'VerticalLayout {\n                spacing: 16px;');

    assert.match(tablesSection, /ScrollView \{/);
    assert.match(tablesSection, /vertical-scrollbar-policy: ScrollBarPolicy\.as-needed/);
    assert.match(tablesSection, /horizontal-scrollbar-policy: ScrollBarPolicy\.always-off/);
    assert.doesNotMatch(tablesSection, /ScrollBarPolicy\.always-on/);
  });

  it("uses one real global ScrollView for the right-side scrollbar", () => {
    const globalSection = segmentBetween(
      desktopUi,
      '        ScrollView {\n            vertical-stretch: 1;',
      '\n    }\n}'
    );

    assert.match(globalSection, /vertical-scrollbar-policy: ScrollBarPolicy\.always-on/);
    assert.match(globalSection, /horizontal-scrollbar-policy: ScrollBarPolicy\.always-off/);
    assert.match(globalSection, /HorizontalLayout \{/);
    assert.match(globalSection, /preferred-height: 700px/);
    assert.match(globalSection, /Panel \{/);
  });

  it("keeps the header minimal and removes the footer", () => {
    assert.equal(desktopUi.includes('text: "Schema Pathfinder";'), true);
    assert.doesNotMatch(desktopUi, /Local FK path inspector/);
    assert.doesNotMatch(desktopUi, /Fullscreen/);
    assert.doesNotMatch(desktopUi, /toggle-fullscreen/);
    assert.doesNotMatch(desktopUi, /text: root.status/);
    assert.doesNotMatch(desktopUi, /height: 34px/);
  });

  it("has exactly one always-visible vertical scrollbar", () => {
    const alwaysOnCount = countMatches(desktopUi, /vertical-scrollbar-policy: ScrollBarPolicy\.always-on/g);

    assert.equal(alwaysOnCount, 1);
  });
});
