export const packageManifest = {
  productName: "Schema Pathfinder",
  packageName: "schema-pathfinder",
  version: "0.1.2",
  description: "CLI for finding declared PostgreSQL foreign-key paths between tables.",
  maintainer: "Dress-Shot <contact@dress-shot.app>",
  homepage: "https://github.com/Dress-Shot/schema-pathfinder",
  installRoot: "/opt/schema-pathfinder",
  cliDepends: "libc6 (>= 2.31), libgcc-s1",
  desktopDepends: [
    "libc6 (>= 2.31)",
    "libgcc-s1",
    "libfontconfig1",
    "libfreetype6",
    "libxkbcommon0",
    "libxkbcommon-x11-0",
    "libx11-6",
    "libx11-xcb1",
    "libxcb1",
    "libgl1",
    "libegl1",
    "libwayland-client0",
    "libwayland-cursor0",
    "libwayland-egl1"
  ].join(", ")
};
