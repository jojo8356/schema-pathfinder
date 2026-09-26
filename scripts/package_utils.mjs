import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { homedir } from "node:os";
import { packageManifest } from "./package_manifest.mjs";

export { packageManifest };

export function repoRoot() {
  return fileURLToPath(new URL("..", import.meta.url));
}

export function distRoot() {
  return join(repoRoot(), "dist");
}

export function resetDirectory(path) {
  rmSync(path, {
    recursive: true,
    force: true
  });
  mkdirSync(path, {
    recursive: true
  });
}

export function ensureDirectory(path) {
  mkdirSync(path, {
    recursive: true
  });
}

export function copyRuntimeTree(targetRoot) {
  const root = repoRoot();
  const includedPaths = [
    "fixtures",
    "README.md"
  ];

  for (const relativePath of includedPaths) {
    const source = join(root, relativePath);

    if (existsSync(source) === false) {
      throw new Error(`Missing package input: ${relativePath}`);
    }

    const target = join(targetRoot, relativePath);
    ensureDirectory(dirname(target));
    cpSync(source, target, {
      recursive: true,
      dereference: true
    });
  }
}

export function buildRustCliRelease() {
  execFileSync("cargo", ["build", "--release", "-p", "schema-pathfinder", "--bin", "schema-pathfinder"], {
    cwd: repoRoot(),
    stdio: "inherit"
  });
}

export function rustCliBinaryPath() {
  return join(repoRoot(), "target", "release", "schema-pathfinder");
}

export function copyRustCliBinary(targetPath) {
  const source = rustCliBinaryPath();

  if (existsSync(source) === false) {
    throw new Error("Rust CLI binary is missing. Run cargo build --release -p schema-pathfinder --bin schema-pathfinder.");
  }

  ensureDirectory(dirname(targetPath));
  copyFileSync(source, targetPath);
  chmodSync(targetPath, 0o755);
}

export function writeExecutable(path, content) {
  writeFileSync(path, content);
  chmodSync(path, 0o755);
}

export function createCliLauncher(binaryPath) {
  return `#!/bin/sh
set -eu
exec "${binaryPath}" "$@"
`;
}

export function createDesktopFile(executableName) {
  return `[Desktop Entry]
Type=Application
Name=${packageManifest.productName}
Comment=${packageManifest.description}
Exec=${executableName} --help
Icon=${packageManifest.packageName}
Terminal=true
Categories=Development;Database;
`;
}

export function createSvgIcon() {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="42" fill="#15181e"/>
  <path d="M70 74h46v30H70zM140 74h46v30h-46zM105 152h46v30h-46z" fill="#f4f7fb"/>
  <path d="M116 89h24M128 104v48M151 167h24M175 104v63" stroke="#5eead4" stroke-width="14" stroke-linecap="round" fill="none"/>
</svg>
`;
}


export function appImageToolArgs(appDir, artifactPath) {
  const runtimeFile = process.env.APPIMAGE_RUNTIME_FILE || "/tmp/runtime-x86_64";

  if (existsSync(runtimeFile) === true) {
    return ["--runtime-file", runtimeFile, appDir, artifactPath];
  }

  return [appDir, artifactPath];
}

export function findAppImageTool() {
  const candidates = [];

  if (process.env.APPIMAGETOOL) {
    candidates.push(process.env.APPIMAGETOOL);
  }

  const home = homedir();

  if (home) {
    candidates.push(join(home, ".local", "bin", "appimagetool"));
  }

  candidates.push("/usr/local/bin/appimagetool", "/usr/bin/appimagetool");

  for (const candidate of candidates) {
    if (existsSync(candidate) === true) {
      return candidate;
    }
  }

  throw new Error(
    "appimagetool not found. Install it, put it in ~/.local/bin, or set APPIMAGETOOL to its path."
  );
}

export function assertRuntimeReady() {
  execFileSync("cargo", ["--version"], {
    stdio: "ignore"
  });
}
