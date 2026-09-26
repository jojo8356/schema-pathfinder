import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const workflow = readFileSync(".github/workflows/build-binaries.yml", "utf8");
const manifest = readFileSync("scripts/package_manifest.mjs", "utf8");
const cliDeb = readFileSync("scripts/build_deb.mjs", "utf8");
const desktopDeb = readFileSync("scripts/build_desktop_deb.mjs", "utf8");

describe("release workflow", () => {
  it("publishes on tags and on manual dispatch", () => {
    assert.match(workflow, /workflow_dispatch:/);
    assert.match(workflow, /release_tag:/);
    assert.match(workflow, /tags:\n\s+- "v\*"/);
    assert.match(workflow, /contents: write/);
  });

  it("builds both Debian packages", () => {
    assert.match(workflow, /node scripts\/build_deb\.mjs/);
    assert.match(workflow, /node scripts\/build_desktop_deb\.mjs/);
  });

  it("builds both AppImages", () => {
    assert.match(workflow, /node scripts\/build_appimage\.mjs/);
    assert.match(workflow, /node scripts\/build_desktop_appimage\.mjs/);
    assert.match(workflow, /appimagetool/);
  });

  it("verifies the packages before publishing them", () => {
    assert.match(workflow, /dpkg-deb --info/);
    assert.match(workflow, /apt-get install --yes \.\/dist\/schema-pathfinder_\*_amd64\.deb/);
    assert.match(workflow, /apt-get install --yes \.\/dist\/schema-pathfinder-desktop_\*_amd64\.deb/);
    assert.match(workflow, /AppImage --tables --fixture/);
  });

  it("uploads a complete asset set to the release", () => {
    assert.match(workflow, /cp dist\/\*\.deb artifacts\//);
    assert.match(workflow, /cp dist\/\*\.AppImage artifacts\//);
    assert.match(workflow, /tar -C artifacts -czf artifacts\/schema-pathfinder-linux-x86_64\.tar\.gz/);
    assert.match(workflow, /sha256sum artifacts\/\* > artifacts\/SHA256SUMS/);
    assert.match(workflow, /gh release upload "\$TAG" artifacts\/\* --repo "\$GITHUB_REPOSITORY" --clobber/);
    assert.match(workflow, /gh release create "\$TAG"/);
  });

  it("declares runtime dependencies for both Debian packages", () => {
    assert.match(manifest, /cliDepends:/);
    assert.match(manifest, /desktopDepends:/);
    assert.match(manifest, /libfontconfig1/);
    assert.match(manifest, /libxkbcommon0/);
    assert.match(cliDeb, /Depends: \$\{packageManifest\.cliDepends\}/);
    assert.match(desktopDeb, /"Depends: " \+ packageManifest\.desktopDepends/);
  });

  it("builds root owned Debian packages", () => {
    assert.match(cliDeb, /"--root-owner-group", "--build"/);
    assert.match(desktopDeb, /"--root-owner-group", "--build"/);
  });

  it("keeps every packaged version in sync", () => {
    const version = manifest.match(/version: "([^"]+)"/)[1];
    const rootPackage = JSON.parse(readFileSync("package.json", "utf8"));
    const cliCargo = readFileSync("apps/cli-rust/Cargo.toml", "utf8");
    const desktopCargo = readFileSync("apps/desktop/Cargo.toml", "utf8");

    assert.equal(rootPackage.version, version);
    assert.match(cliCargo, new RegExp('version = "' + version + '"'));
    assert.match(desktopCargo, new RegExp('version = "' + version + '"'));
  });
});
