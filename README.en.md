<div align="center">

# Schema Pathfinder

**Instantly find how to connect two PostgreSQL tables through their foreign keys.**

Rust CLI · Slint desktop UI · React web UI · Rust API · fixtures & contract tests

[![CI](https://github.com/jojo8356/schema-pathfinder/actions/workflows/build-binaries.yml/badge.svg)](https://github.com/jojo8356/schema-pathfinder/actions/workflows/build-binaries.yml)
![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)
![Node](https://img.shields.io/badge/Node-%3E%3D22-339933?logo=nodedotjs&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-metadata%20only-4169E1?logo=postgresql&logoColor=white)
![Tests](https://img.shields.io/badge/tests-131%20(66%20Node%20%2B%2065%20Rust)-brightgreen)
![Status](https://img.shields.io/badge/version-0.1.3-blue)

</div>

> 🇫🇷 A French version of this README is available in [README.md](README.md).

---

## Table of contents

- [Overview](#overview)
- [The problem it solves](#the-problem-it-solves)
- [Features](#features)
- [Quick start](#quick-start)
- [Available surfaces](#available-surfaces)
- [Output formats](#output-formats)
- [CLI usage](#cli-usage)
- [Web API](#web-api)
- [Architecture](#architecture)
- [Development](#development)
- [Testing and quality](#testing-and-quality)
- [API documentation (rustdoc)](#api-documentation-rustdoc)
- [Packaging and distribution](#packaging-and-distribution)
- [Security and privacy](#security-and-privacy)
- [Full documentation](#full-documentation)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**Schema Pathfinder** is a standalone tool that answers a very concrete,
day-to-day question for developers, analysts, and DBAs:

> _"How do I join table A to table B?"_

The tool reads **only the metadata** of a PostgreSQL schema — tables, schemas,
and declared foreign keys — then computes and ranks the possible join paths
between two tables. It never reads the business data stored in the rows.

A single Rust core (`schema_pathfinder::pathfinder_core`) powers **four
surfaces**: a CLI, a desktop application, a web interface, and a lightweight HTTP
API.

## The problem it solves

In a real database, two tables are often connected by **several foreign-key
paths**. The highest-"scored" path is not always the one you want: it may go
through a table that is still **empty at data-entry time** (for example, a
billing table not yet populated when writing a delivery note).

Schema Pathfinder addresses this by listing **all the paths**, **ranked from
simplest to most complex** (2 links, then 3, then 4, then 5…), with a
**user-settable limit on the number of links** (`--max-links`, default `5`). The
user sees all the alternatives and picks the one that fits their context.

## Features

- 🔎 **Foreign-key path search** between two tables, ranked by increasing number of links.
- 🎚️ **User-settable link limit** (`--max-links`, default 5) on the CLI, desktop, and web.
- 🌳 **Full architecture tree**: databases → schemas → tables → FK relationships.
- 📋 **Listing** of visible tables, schemas, and databases.
- 🔌 **Three data sources**: JSON fixture, SQL DDL file, or a live PostgreSQL connection.
- 🧩 **Five output formats**: `text`, `equation`, `sql`, `mermaid`, `json`.
- 🔄 **DDL → JSON fixture conversion**, reusable, without a running database.
- 🖥️ **Desktop application** (Slint), local, with no server or backend URL.
- 🌐 **Web interface** (React) + a lightweight **Rust API** (`tiny_http`).
- 📦 **Packaging** ready to distribute: `.deb` and `.AppImage` (CLI and desktop).

## Quick start

Requirements: **Node.js 22+**, **pnpm 11+**, **stable Rust**, and **Cargo**.

```bash
# 1. Clone and install
git clone https://github.com/jojo8356/schema-pathfinder.git
cd schema-pathfinder
pnpm install

# 2. Build the CLI
cargo build --release -p schema-pathfinder --bin schema-pathfinder

# 3. Find a path between two tables (using an example fixture)
./target/release/schema-pathfinder path ClothingItem User \
  --format equation \
  --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```

Output (`equation` format):

```text
public.ClothingItem.sellerProfileId = public.SellerProfile.id
->
public.SellerProfile.userId = public.User.id
```

## Available surfaces

| Surface        | Technology                       | Primary use                                              |
| -------------- | -------------------------------- | ------------------------------------------------------- |
| **CLI**        | Rust, `lexopt`                   | Terminal, scripts, quick audit, CI, offline mode.       |
| **Desktop UI** | Rust, Slint                      | Native local app, with no server or backend URL.        |
| **Web UI**     | React 19, Vite, Lucide           | Visual exploration in the browser.                      |
| **Web API**    | Rust, `tiny_http`                | Lightweight HTTP server that powers the web UI.         |
| **Contracts**  | JSON fixtures + Node/Rust tests  | Stable validation of formats and behaviors.             |

## Output formats

The format is chosen with `--format <name>`. The five supported formats:

| Format     | Description                                                      |
| ---------- | --------------------------------------------------------------- |
| `text`     | Readable summary: score, evidence, path length.                 |
| `equation` | Sequence of `A.col = B.col` equalities linking the tables.      |
| `sql`      | Ready-to-run `SELECT … JOIN …` query (quoted identifiers).      |
| `mermaid`  | `flowchart LR` diagram for documentation.                       |
| `json`     | Structured output, ideal for tooling integration.              |

## CLI usage

```text
Usage: schema-pathfinder <command> [options]

Commands:
  tables                   List tables (fixture, SQL, or PostgreSQL metadata)
  databases, dbs           List PostgreSQL databases
  schemas                  List schemas of the selected database
  tree                     Print visible databases, schemas, tables, and FKs
  fixture                  Generate the JSON fixture from the chosen source
  path <source> <target>   List FK paths between two tables, simplest first

Options:
  --tables | --databases,--dbs | --schemas | --tree   command shortcuts
  --database-url <url>     read metadata from this PostgreSQL connection
  --fixture <path>         read FKs from a JSON fixture
  --sql <path>             read a PostgreSQL DDL and convert it to metadata
  --max-links <n>          max number of FK links per listed path (default 5)
  --format <fmt>           text | equation | sql | mermaid | json
  -h, --help               display help

Environment:
  DATABASE_URL             connection string used when no explicit source is provided
```

Examples:

```bash
# List the tables of a fixture
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json

# Introspect a live PostgreSQL database
schema-pathfinder --tree      --database-url postgres://readonly:secret@localhost:5432/postgres
schema-pathfinder --schemas   --database-url postgres://readonly:secret@localhost:5432/postgres
schema-pathfinder --databases --database-url postgres://readonly:secret@localhost:5432/postgres

# Paths between two tables, capped at 3 links, in SQL
schema-pathfinder path ClothingItem User --max-links 3 --format sql \
  --fixture fixtures/postgres/dressshot_seed_fk_edges.json

# Convert a SQL DDL into a JSON fixture
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

## Web API

The Rust API serves the built web UI and exposes JSON endpoints:

| Method  | Endpoint         | Description                                       |
| ------- | ---------------- | ------------------------------------------------ |
| `GET`   | `/api/health`    | Liveness probe.                                  |
| `GET`   | `/api/tables`    | List the tables of the configured source.        |
| `GET`   | `/api/databases` | List PostgreSQL databases.                        |
| `GET`   | `/api/schemas`   | List the schemas of the selected database.        |
| `POST`  | `/api/path`      | FK paths between two tables (source, target…).    |

```bash
# Build and run the whole web stack (UI + API)
pnpm build:saas
cargo run --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## Architecture

```
schema-pathfinder/
├── apps/
│   ├── cli-rust/            # Rust core: engine, CLI, and API
│   │   ├── src/
│   │   │   ├── pathfinder_core.rs        # shared engine (graph, scoring, renderers)
│   │   │   ├── main.rs                   # CLI binary (lexopt)
│   │   │   ├── postgres_sql_metadata.rs  # PostgreSQL DDL parsing
│   │   │   └── bin/schema-pathfinder-api.rs  # HTTP API (tiny_http)
│   │   └── tests/           # Rust integration tests (public API)
│   ├── desktop/             # Rust + Slint desktop UI
│   └── web/                 # React + Vite web UI
├── packages/
│   ├── core/                # JS mirror of the engine (tests & contracts)
│   └── contracts/           # shared contract types
├── fixtures/postgres/       # example schemas and test expectations
├── scripts/                 # .deb / .AppImage builds, checks
├── tests/                   # Node tests (contracts, behaviors)
└── docs/                    # detailed documentation
```

Guiding principle: **a single implementation of the business logic in Rust**
(`pathfinder_core`), reused by the CLI, the API, and the desktop. A JavaScript
mirror under `packages/core` acts as a contract test bench and verifies the
stability of the formats.

## Development

```bash
pnpm install                                   # workspace dependencies
pnpm test                                       # Node tests (66)
pnpm lint                                        # scaffold + contracts
pnpm typecheck                                    # structure check
cargo test -p schema-pathfinder                   # Rust tests (65)
cargo build --release -p schema-pathfinder        # optimized CLI build
```

## Testing and quality

The project is covered by **131 tests**: **66 Node tests** (behaviors and
contracts) and **65 Rust tests** (unit + integration).

- Rust unit tests: `#[cfg(test)] mod tests` inside the modules.
- Rust integration tests: one file per domain under `apps/cli-rust/tests/`
  (`path_search`, `scoring`, `renderers`, `sql_metadata`, `architecture_tree`, …).
- Node tests: `tests/*.test.mjs` (run via the built-in `node --test` runner).
- Doctests: the Rust documentation examples are executed by `cargo test`.

A **complete tutorial** explains how to write all of these tests, measure
coverage (`cargo llvm-cov`), and tool the CLI: see
[docs/testing-complete-tutorial.en.md](docs/testing-complete-tutorial.en.md).

## API documentation (rustdoc)

Every public item of the Rust core is documented. You can generate the full HTML
site — one page per module, type, and function, with built-in search:

```bash
pnpm doc:rust          # generate the site in target/doc/
pnpm doc:rust:open     # generate and open target/doc/schema_pathfinder/index.html
# direct equivalent:
cargo doc --no-deps -p schema-pathfinder --open
```

Entry point: `target/doc/schema_pathfinder/index.html`.

## Packaging and distribution

```bash
node scripts/build_deb.mjs               # CLI .deb
node scripts/build_appimage.mjs          # CLI .AppImage
node scripts/build_desktop_deb.mjs       # desktop .deb
node scripts/build_desktop_appimage.mjs  # desktop .AppImage
```

The artifacts are produced in `dist/`:

- `schema-pathfinder_0.1.3_amd64.deb`
- `schema-pathfinder-0.1.3-x86_64.AppImage`
- `schema-pathfinder-desktop_0.1.3_amd64.deb`
- `schema-pathfinder-desktop-0.1.3-x86_64.AppImage`

Installing from a release (artifacts built and verified by CI, with `SHA256SUMS`):

```bash
# Debian packages
sudo apt-get install ./schema-pathfinder_0.1.3_amd64.deb
sudo apt-get install ./schema-pathfinder-desktop_0.1.3_amd64.deb

# or AppImage, no installation
chmod +x schema-pathfinder-desktop-0.1.3-x86_64.AppImage
./schema-pathfinder-desktop-0.1.3-x86_64.AppImage
```

## Security and privacy

Schema Pathfinder inspects the **structure** of a database, never the **business
rows**. The documentation examples use dummy URLs. No password, local dump, or
private data should be committed. **Read-only** access is enough for
introspection. See [docs/security-and-privacy.en.md](docs/security-and-privacy.en.md).

## Full documentation

- [Employer overview](docs/employer-overview.en.md)
- [Architecture](docs/architecture.en.md)
- [CLI reference](docs/cli-reference.en.md)
- [API reference](docs/api-reference.en.md)
- [UI guide](docs/ui-guide.en.md)
- [Deployment and packaging](docs/deployment.en.md)
- [Testing and quality](docs/testing-and-quality.en.md)
- [Complete tutorial: writing all the tests](docs/testing-complete-tutorial.en.md)
- [Security and privacy](docs/security-and-privacy.en.md)
- [Roadmap](docs/roadmap.en.md)
- [Contributing](CONTRIBUTING.en.md)

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.en.md](CONTRIBUTING.en.md)
and make sure `pnpm test`, `pnpm lint`, and `cargo test -p schema-pathfinder`
pass before any pull request.

## License

No license has been declared for this repository yet. Until a `LICENSE` file is
added, the code remains "all rights reserved" by default. Contact the maintainer
for any use.
