import { chmodSync, copyFileSync, existsSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import {
  assertRuntimeReady,
  copyRuntimeTree,
  createSvgIcon,
  distRoot,
  ensureDirectory,
  packageManifest,
  repoRoot,
  resetDirectory,
  writeExecutable
} from "./package_utils.mjs";

const desktopPackageName = packageManifest.packageName + "-desktop";

assertRuntimeReady();
buildRustDesktopRelease();

const dist = distRoot();
const appDir = join(dist, "appimage", desktopPackageName + ".AppDir");
const appRoot = join(appDir, "usr", "app");
const binaryPath = join(appDir, "usr", "bin", desktopPackageName);
const artifactPath = join(dist, desktopPackageName + "-" + packageManifest.version + "-x86_64.AppImage");

resetDirectory(appDir);
ensureDirectory(appRoot);
ensureDirectory(join(appDir, "usr", "bin"));

copyRuntimeTree(appRoot);
copyRustDesktopBinary(binaryPath);

writeExecutable(join(appDir, "AppRun"), createDesktopLauncher("$APPDIR/usr/bin/" + desktopPackageName));
writeFileSync(join(appDir, desktopPackageName + ".desktop"), createDesktopFile(desktopPackageName));
writeFileSync(join(appDir, desktopPackageName + ".svg"), createSvgIcon());
copyFileSync(join(appDir, desktopPackageName + ".svg"), join(appDir, ".DirIcon"));

const appImageTool = findAppImageTool();
execFileSync(appImageTool, [appDir, artifactPath], {
  cwd: repoRoot(),
  env: {
    ...process.env,
    ARCH: "x86_64"
  },
  stdio: "inherit"
});

console.log(artifactPath);

function buildRustDesktopRelease() {
  execFileSync("cargo", ["build", "--release", "-p", "schema-pathfinder-desktop"], {
    cwd: repoRoot(),
    stdio: "inherit"
  });
}

function rustDesktopBinaryPath() {
  return join(repoRoot(), "target", "release", "schema-pathfinder-desktop");
}

function copyRustDesktopBinary(targetPath) {
  const source = rustDesktopBinaryPath();

  if (existsSync(source) === false) {
    throw new Error("Rust desktop binary is missing. Run cargo build --release -p schema-pathfinder-desktop.");
  }

  ensureDirectory(dirname(targetPath));
  copyFileSync(source, targetPath);
  chmodSync(targetPath, 0o755);
}

function createDesktopLauncher(binaryPath) {
  return "#!/bin/sh\n" +
    "set -eu\n" +
    "cd \"$APPDIR/usr/app\"\n" +
    "exec \"" + binaryPath + "\" \"$@\"\n";
}

function createDesktopFile(executableName) {
  return "[Desktop Entry]\n" +
    "Type=Application\n" +
    "Name=" + packageManifest.productName + "\n" +
    "Comment=Desktop UI for finding declared PostgreSQL foreign-key paths between tables.\n" +
    "Exec=" + executableName + "\n" +
    "Icon=" + executableName + "\n" +
    "Terminal=false\n" +
    "Categories=Development;Database;\n";
}

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
