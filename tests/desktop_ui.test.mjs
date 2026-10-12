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
    assert.equal(pathSection.includes("options: root.schema-options;"), true);
    assert.equal(pathSection.includes("options: root.table-options;"), true);
    assert.equal(pathSection.includes("root.select-schema(value);"), true);
    assert.equal(pathSection.includes("value <=> root.source-table;"), true);
    assert.equal(pathSection.includes("value <=> root.target-table;"), true);
    assert.doesNotMatch(pathSection, /LineEdit/);
  });

  it("defines the drop-downs once and reuses them in both layouts", () => {
    assert.match(desktopUi, /component TableSelector inherits ComboBox \{/);
    assert.match(desktopUi, /component FormatSelector inherits ComboBox \{/);
    assert.match(desktopUi, /component MaxLinksBox inherits SpinBox \{/);
    // Three selectors (schema, from, to) in each of the two layouts.
    assert.equal(countMatches(desktopUi, /TableSelector \{/g), 6);
    assert.equal(countMatches(desktopUi, /FormatSelector \{/g), 2);
    assert.equal(countMatches(desktopUi, /MaxLinksBox \{/g), 2);
  });

  it("keeps list scrollbars hidden until content overflows", () => {
    const listSection = segmentBetween(desktopUi, "component MonospaceList inherits ScrollView {", "component TableSelector");

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
    assert.match(desktopUi, /property <bool> compact: root\.width < 900px;/);
    assert.match(desktopUi, /if !root\.compact: Rectangle \{/);
    assert.match(desktopUi, /if root\.compact: ScrollView \{/);
    assert.match(desktopUi, /viewport-height: Math\.max\(self\.visible-height, \d+px\);/);
    assert.match(desktopUi, /property <length> header-height: root\.compact \? 48px : 56px;/);
  });

  it("keeps the workspace usable on a small window", () => {
    const windowProperties = windowSection();

    assert.match(windowProperties, /min-width: 420px;/);
    assert.match(windowProperties, /min-height: 380px;/);

    // Every panel must be able to shrink, otherwise a narrow window clips them.
    assert.doesNotMatch(desktopUi, /min-width: 320px/);
    assert.doesNotMatch(desktopUi, /min-width: 280px/);

    // Narrow layout stacks the selectors, one per line.
    const stackedSection = segmentBetween(
      desktopUi,
      "// Narrow screens: one selector per line",
      "// Wide screens: format, max links"
    );

    assert.equal(countMatches(stackedSection, /TableSelector \{/g), 3);
    assert.equal(countMatches(stackedSection, /Row \{/g), 3);
    assert.doesNotMatch(stackedSection, /min-width: 120px/);
  });

  it("caps and centres the workspace on large screens", () => {
    assert.match(desktopUi, /property <length> content-max-width: 1800px;/);
    assert.match(desktopUi, /width: Math\.min\(parent\.width, root\.content-max-width\);/);
    assert.match(desktopUi, /x: \(parent\.width - self\.width\) \/ 2;/);
    // The sidebar keeps a readable width instead of stretching with the window.
    assert.match(desktopUi, /preferred-width: 340px;/);
    assert.match(desktopUi, /max-width: 440px;/);
  });

  it("never derives a horizontal size, padding or font from the window width", () => {
    // Slint sizes the window from the layout info, so any horizontal size,
    // padding or font-size bound to root.width closes a binding loop that
    // "may cause panic at runtime". Only `if` branches and vertical sizes may
    // depend on root.width.
    assert.doesNotMatch(desktopUi, /font-size: root\./);
    assert.doesNotMatch(desktopUi, /padding[a-z-]*: root\./);
    assert.doesNotMatch(desktopUi, /(?:min|preferred|max)-width: root\./);
    assert.doesNotMatch(desktopUi, /width: root\.compact/);
    assert.match(desktopUi, /height: root\.header-height;/);
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

  it("lets the user pick a maximum number of links and lists ranked paths", () => {
    assert.match(desktopUi, /import \{[^}]*SpinBox[^}]*\} from "std-widgets.slint";/);
    assert.match(desktopUi, /text: "Max links";/);
    assert.match(desktopUi, /SpinBox \{/);
    assert.match(desktopUi, /value <=> root\.max-links;/);
    assert.match(desktopUi, /callback find-path\(string, string, string, string, int\);/);
    assert.match(desktopMain, /ranked_paths_by_complexity/);
    assert.match(desktopMain, /render_paths/);
    assert.match(desktopMain, /set_max_links/);
    assert.doesNotMatch(desktopMain, /best_path/);
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
