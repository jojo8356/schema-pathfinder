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
      '        Rectangle {\n            height: 34px;'
    );

    assert.match(globalSection, /vertical-scrollbar-policy: ScrollBarPolicy\.always-on/);
    assert.match(globalSection, /horizontal-scrollbar-policy: ScrollBarPolicy\.always-off/);
    assert.match(globalSection, /HorizontalLayout \{/);
    assert.match(globalSection, /preferred-height: 700px/);
    assert.match(globalSection, /Panel \{/);
  });

  it("has exactly one always-visible vertical scrollbar", () => {
    const alwaysOnCount = countMatches(desktopUi, /vertical-scrollbar-policy: ScrollBarPolicy\.always-on/g);

    assert.equal(alwaysOnCount, 1);
  });
});
