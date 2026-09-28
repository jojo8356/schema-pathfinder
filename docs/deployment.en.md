# Deployment and Packaging

## Distribution modes

Schema Pathfinder can be shipped in several forms:

- Standalone Rust CLI.
- AppImage desktop application.
- Debian desktop application.
- Rust API alone.
- Web + Rust API bundle via Docker.

## Build the CLI

```bash
cargo build --release -p schema-pathfinder --bin schema-pathfinder
```

Generated binary:

```text
target/release/schema-pathfinder
```

## Build the API

```bash
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

Generated binary:

```text
target/release/schema-pathfinder-api
```

## Build the web UI

```bash
pnpm build:web
```

## Linux packaging

```bash
node scripts/build_deb.mjs
node scripts/build_appimage.mjs
node scripts/build_desktop_deb.mjs
node scripts/build_desktop_appimage.mjs
```

The scripts produce the artifacts in `dist/`.

The Debian packages declare their runtime dependencies:

- `schema-pathfinder`: `libc6`, `libgcc-s1`.
- `schema-pathfinder-desktop`: Fontconfig, X11/XCB, xkbcommon, Wayland, and GL libraries required by the Slint runtime.

## GitHub release

The `.github/workflows/build-binaries.yml` workflow builds the binaries, generates the two `.deb` packages and the two AppImages, verifies their installation (`apt-get install ./dist/*.deb`) and their execution (`AppImage --tables`), then publishes everything to the release.

Two ways to publish:

```bash
# 1. publication triggered by a tag
git tag v0.1.3
git push origin v0.1.3

# 2. manual publication to an existing release
gh workflow run build-binaries.yml --ref <branch> --field release_tag=v0.1.3
```

Assets published on the release:

```text
schema-pathfinder-linux-x86_64
schema-pathfinder-desktop-linux-x86_64
schema-pathfinder-linux-x86_64.tar.gz
schema-pathfinder_<version>_amd64.deb
schema-pathfinder-desktop_<version>_amd64.deb
schema-pathfinder-<version>-x86_64.AppImage
schema-pathfinder-desktop-<version>-x86_64.AppImage
SHA256SUMS
```

`appimagetool` is resolved via `APPIMAGETOOL`, `~/.local/bin`, `/usr/local/bin`, then `/usr/bin`. In CI it is installed in `/usr/local/bin` and run with `APPIMAGE_EXTRACT_AND_RUN=1`, the runtime being provided by `APPIMAGE_RUNTIME_FILE`.

Installing from the release:

```bash
sudo apt-get install ./schema-pathfinder_0.1.3_amd64.deb
sudo apt-get install ./schema-pathfinder-desktop_0.1.3_amd64.deb
```

## Docker

The Dockerfile builds the web UI and then the Rust API. This mode is intended for controlled exposure behind a reverse proxy or a deployment platform.

Things to check in production:

- `DATABASE_URL` provided at runtime if the API must inspect a default database.
- `SCHEMA_PATHFINDER_API_TOKEN` enabled if the instance is reachable from the internet.
- Public URL and CORS adapted to the environment.
- Network access from the container to PostgreSQL.

## Important rule

The CLI and desktop must not depend on a backend URL. They are distributed as independent local tools.
