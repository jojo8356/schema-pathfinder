#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { findPaths } from "../../core/src/path_search.mjs";
import { renderText } from "../../core/src/renderers.mjs";

export function runCli(argv, io = defaultIo()) {
  const [command, sourceTable, targetTable] = argv;

  if (command !== "path") {
    return usage(io, "Expected command: path");
  }

  if (sourceTable === undefined || targetTable === undefined) {
    return usage(io, "Missing source or target table");
  }

  const fixture = JSON.parse(readFileSync("fixtures/postgres/dressshot_seed_fk_edges.json", "utf8"));
  const result = findPaths({
    edges: fixture.edges,
    sourceTable,
    targetTable
  });

  if (result.paths.length === 0) {
    io.stdout(`${result.noPathReason}\n`);
    return 0;
  }

  io.stdout(`${renderText(result.paths[0])}\n`);
  return 0;
}

function usage(io, message) {
  io.stderr(`${message}\n\nUsage: schema-pathfinder path <source-table> <target-table>\n`);
  return 1;
}

function defaultIo() {
  return {
    stdout(value) {
      process.stdout.write(value);
    },
    stderr(value) {
      process.stderr.write(value);
    }
  };
}

if (fileURLToPath(import.meta.url) === process.argv[1]) {
  const exitCode = runCli(process.argv.slice(2));
  process.exit(exitCode);
}
