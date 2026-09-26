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
const desktopInstallRoot = packageManifest.installRoot + "-desktop";

assertRuntimeReady();
buildRustDesktopRelease();

const dist = distRoot();
const packageRoot = join(dist, "deb", desktopPackageName + "_" + packageManifest.version + "_amd64");
const appRoot = join(packageRoot, desktopInstallRoot);
const debianRoot = join(packageRoot, "DEBIAN");
const binRoot = join(packageRoot, "usr", "bin");
const applicationsRoot = join(packageRoot, "usr", "share", "applications");
const iconsRoot = join(packageRoot, "usr", "share", "icons", "hicolor", "scalable", "apps");
const artifactPath = join(dist, desktopPackageName + "_" + packageManifest.version + "_amd64.deb");

resetDirectory(packageRoot);
ensureDirectory(appRoot);
ensureDirectory(debianRoot);
ensureDirectory(binRoot);
ensureDirectory(applicationsRoot);
ensureDirectory(iconsRoot);

copyRuntimeTree(appRoot);
copyRustDesktopBinary(join(appRoot, "bin", desktopPackageName));
writeExecutable(join(binRoot, desktopPackageName), createDesktopLauncher(desktopInstallRoot + "/bin/" + desktopPackageName));
writeFileSync(join(applicationsRoot, desktopPackageName + ".desktop"), createDesktopFile(desktopPackageName));
writeFileSync(join(iconsRoot, desktopPackageName + ".svg"), createSvgIcon());
writeControlFile(debianRoot, desktopPackageName);

chmodSync(debianRoot, 0o755);

execFileSync("dpkg-deb", ["--root-owner-group", "--build", packageRoot, artifactPath], {
  cwd: repoRoot(),
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
    "cd \"" + desktopInstallRoot + "\"\n" +
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

function writeControlFile(debianRoot, packageName) {
  writeFileSync(
    join(debianRoot, "control"),
    "Package: " + packageName + "\n" +
      "Version: " + packageManifest.version + "\n" +
      "Section: utils\n" +
      "Priority: optional\n" +
      "Architecture: amd64\n" +
      "Depends: " + packageManifest.desktopDepends + "\n" +
      "Maintainer: " + packageManifest.maintainer + "\n" +
      "Homepage: " + packageManifest.homepage + "\n" +
      "Description: " + packageManifest.productName + " desktop UI\n" +
      " Desktop application for finding declared PostgreSQL foreign-key paths and rendering results.\n"
  );
}
