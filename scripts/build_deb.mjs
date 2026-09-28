import { chmodSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import {
  assertRuntimeReady,
  buildRustCliRelease,
  copyRustCliBinary,
  copyRuntimeTree,
  createCliLauncher,
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
const packageRoot = join(dist, "deb", `${packageManifest.packageName}_${packageManifest.version}_amd64`);
const appRoot = join(packageRoot, packageManifest.installRoot);
const debianRoot = join(packageRoot, "DEBIAN");
const binRoot = join(packageRoot, "usr", "bin");
const artifactPath = join(dist, `${packageManifest.packageName}_${packageManifest.version}_amd64.deb`);

resetDirectory(packageRoot);
ensureDirectory(appRoot);
ensureDirectory(debianRoot);
ensureDirectory(binRoot);

copyRuntimeTree(appRoot);
copyRustCliBinary(join(appRoot, "bin", packageManifest.packageName));

writeExecutable(
  join(binRoot, packageManifest.packageName),
  createCliLauncher(`${packageManifest.installRoot}/bin/${packageManifest.packageName}`)
);

writeFileSync(
  join(debianRoot, "control"),
  `Package: ${packageManifest.packageName}
Version: ${packageManifest.version}
Section: utils
Priority: optional
Architecture: amd64
Depends: ${packageManifest.cliDepends}
Maintainer: ${packageManifest.maintainer}
Homepage: ${packageManifest.homepage}
Description: ${packageManifest.description}
 Schema Pathfinder finds declared PostgreSQL foreign-key paths and can render text,
 equation, SQL, JSON, and Mermaid output.
`
);

chmodSync(debianRoot, 0o755);

execFileSync("dpkg-deb", ["--root-owner-group", "--build", packageRoot, artifactPath], {
  cwd: repoRoot(),
  stdio: "inherit"
});

console.log(artifactPath);
