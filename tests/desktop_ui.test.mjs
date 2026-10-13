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

// The guards below inspect code, not prose: the .slint file documents the rules
// it follows, and those comments quote the forbidden bindings.
function withoutComments(source) {
  return source.replace(/\/\/[^\n]*/g, "");
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

  it("defines the drop-downs once and reuses them in the path panel", () => {
    assert.match(desktopUi, /component TableSelector inherits ComboBox \{/);
    assert.match(desktopUi, /component FormatSelector inherits ComboBox \{/);
    assert.match(desktopUi, /component MaxLinksBox inherits SpinBox \{/);
    // Three selectors (schema, from, to) in the single path form.
    assert.equal(countMatches(withoutComments(desktopUi), /TableSelector \{/g), 3);
    assert.equal(countMatches(withoutComments(desktopUi), /FormatSelector \{/g), 1);
    assert.equal(countMatches(withoutComments(desktopUi), /MaxLinksBox \{/g), 1);
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

  it("adapts to every window size through layout constraints only", () => {
    // A resizable Slint window is sized from the layout info of its content, so
    // a width breakpoint inside the layout closes a binding loop. The UI adapts
    // with min/preferred/max widths and stretch factors instead.
    const layout = withoutComments(desktopUi);

    assert.doesNotMatch(layout, /compact/);
    assert.doesNotMatch(layout, /if[^{]*width/);
    assert.match(desktopUi, /min-width: 240px;\s+preferred-width: 340px;\s+max-width: 440px;/);

    // The sidebar keeps its own width; the right column takes whatever is left.
    const workspace = segmentBetween(desktopUi, "// Workspace:", "output-text <=> root.output-text;");
    assert.match(workspace, /SourcePanel \{\s+horizontal-stretch: 0;/);
    assert.match(workspace, /VerticalLayout \{\s+horizontal-stretch: 1;/);
  });

  it("keeps the workspace usable on a small window", () => {
    const windowProperties = windowSection();

    // Sidebar 240px + spacing 16px + result column 340px + padding 32px.
    assert.match(windowProperties, /min-width: 640px;/);
    // Header 56px + fixed path form 360px + one result block, padding included.
    assert.match(windowProperties, /min-height: 610px;/);

    assert.match(desktopUi, /min-width: 240px;/);
    // The result column keeps a floor so the monospace output stays readable
    // next to the sidebar at the window minimum.
    assert.match(desktopUi, /min-width: 340px;/);

    // The path form lists one labelled control per row, so it never needs a
    // narrower variant.
    const formSection = segmentBetween(desktopUi, 'text: "Path";', 'text: "Find path";');

    assert.equal(countMatches(formSection, /TableSelector \{/g), 3);
    assert.equal(countMatches(formSection, /width: 110px;/g), 5);
    assert.equal(countMatches(formSection, /horizontal-stretch: 0;/g), 5);
  });

  it("keeps Path labels fixed and stretches the inputs", () => {
    const formSection = segmentBetween(desktopUi, 'text: "Path";', 'text: "Find path";');
    assert.doesNotMatch(withoutComments(formSection), /GridLayout/);
    assert.match(formSection, /width: 110px;\s+horizontal-stretch: 0;/);
  });

  it("lets Path and Result fill leftover window width", () => {
    const pathPanel = segmentBetween(desktopUi, "component PathPanel inherits Panel {", "component ResultPanel inherits Panel {");
    assert.doesNotMatch(pathPanel, /max-width: 560px;/);
    assert.doesNotMatch(pathPanel, /max-width: 10000px;/);
    const resultPanel = segmentBetween(desktopUi, "component ResultPanel inherits Panel {", "export component AppWindow");
    assert.doesNotMatch(resultPanel, /max-width: 10000px;/);
    assert.doesNotMatch(desktopUi, /content-max-width/);
    assert.doesNotMatch(desktopUi, /Left spacer/);
    assert.doesNotMatch(desktopUi, /Right spacer/);
    assert.match(desktopUi, /preferred-width: 340px;/);
    assert.match(desktopUi, /max-width: 440px;/);
  });

  it("never reads the window geometry back into the layout", () => {
    // Slint sizes the window from the layout info, so any binding - a size, a
    // padding, a font-size, or even the boolean of an `if` - that reads the
    // window geometry closes a loop the compiler reports as "binding loop ...
    // may cause panic at runtime".
    const layout = withoutComments(desktopUi);

    assert.equal(countMatches(layout, /root\.width/g), 0);
    assert.equal(countMatches(layout, /root\.height/g), 0);
    assert.equal(countMatches(layout, /parent\.width/g), 0);
    assert.equal(countMatches(layout, /parent\.height/g), 0);
    assert.equal(countMatches(layout, /self\.width/g), 0);
    assert.equal(countMatches(layout, /self\.height/g), 0);
  });

  it("keeps fixed-size controls out of the stretch distribution", () => {
    // With the default `alignment: stretch`, a BoxLayout sizes stretch-0
    // children at their MINIMUM and, when every factor is 0, shares the leftover
    // equally - which is what inflated the drop-downs on the first screenshot.
    const layout = withoutComments(desktopUi);

    // The form panel is exactly as tall as its content floor, so the column's
    // leftover can only go to the Result panel.
    assert.match(desktopUi, /min-height: 360px;\s+max-height: 360px;/);
    assert.match(desktopUi, /ResultPanel \{\s+vertical-stretch: 1;/);

    // Inside the form, rows keep their preferred height instead of sharing
    // leftover space.
    const formSection = segmentBetween(
      desktopUi,
      "component PathPanel inherits Panel {",
      "component ResultPanel inherits Panel {"
    );
    assert.match(
      withoutComments(formSection),
      /VerticalLayout \{\s+padding: 16px;\s+spacing: 12px;\s+alignment: start;/
    );

    // No layout may opt back into the inflating alignment.
    assert.doesNotMatch(layout, /alignment: stretch/);
  });

  it("expresses conditions with blocks instead of ternary expressions", () => {
    // Slint has no other use for `?`: a ternary hides a branch that the layout
    // and the reader have to resolve inline, and it is how window-size branches
    // keep creeping back in. Conditions are written as `if` blocks.
    const layout = withoutComments(desktopUi);

    assert.equal(countMatches(layout, /\?/g), 0);
    assert.doesNotMatch(layout, /\w\s*\?\s*[^:]+:/);
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
