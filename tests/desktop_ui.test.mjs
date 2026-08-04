import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const desktopUi = readFileSync("apps/desktop/ui/main.slint", "utf8");

describe("desktop UI", () => {
  it("keeps a visible right-side scrollbar for the tables list", () => {
    assert.match(desktopUi, /component TableScrollbar inherits Rectangle/);
    assert.match(desktopUi, /TableScrollbar { }/);
    assert.match(desktopUi, /vertical-scrollbar-policy: ScrollBarPolicy.always-on/);
  });
});
