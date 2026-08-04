import { copyFileSync, existsSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import {
  appImageToolArgs,
  assertRuntimeReady,
  buildRustCliRelease,
  copyRustCliBinary,
  copyRuntimeTree,
  createCliLauncher,
  createDesktopFile,
  createSvgIcon,
  distRoot,
  ensureDirectory,
  packageManifest,
  repoRoot,
  resetDirectory,
  writeExecutable
} from "./package_utils.mjs";

assertRuntimeReady();
buildRustCliRelease();

const dist = distRoot();
const appDir = join(dist, "appimage", `${packageManifest.packageName}.AppDir`);
const appRoot = join(appDir, "usr", "app");
const artifactPath = join(dist, `${packageManifest.packageName}-${packageManifest.version}-x86_64.AppImage`);

resetDirectory(appDir);
ensureDirectory(appRoot);
ensureDirectory(join(appDir, "usr", "bin"));

copyRuntimeTree(appRoot);
copyRustCliBinary(join(appDir, "usr", "bin", packageManifest.packageName));

writeExecutable(
  join(appDir, "AppRun"),
  createCliLauncher(`$APPDIR/usr/bin/${packageManifest.packageName}`)
);

writeFileSync(join(appDir, `${packageManifest.packageName}.desktop`), createDesktopFile("schema-pathfinder"));
writeFileSync(join(appDir, `${packageManifest.packageName}.svg`), createSvgIcon());
copyFileSync(join(appDir, `${packageManifest.packageName}.svg`), join(appDir, ".DirIcon"));

const appImageTool = findAppImageTool();
execFileSync(appImageTool, appImageToolArgs(appDir, artifactPath), {
  cwd: repoRoot(),
  env: {
    ...process.env,
    ARCH: "x86_64"
  },
  stdio: "inherit"
});

console.log(artifactPath);

function findAppImageTool() {
  const candidates = [
    "/home/jojokes/.local/bin/appimagetool",
    "/usr/local/bin/appimagetool",
    "/usr/bin/appimagetool"
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate) === true) {
      return candidate;
    }
  }

  throw new Error("appimagetool not found. Install appimagetool or put it in ~/.local/bin.");
}
