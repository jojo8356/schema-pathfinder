#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Command } from "commander";
import { formatList, parseOutputFormat } from "../../core/src/output_formats.mjs";
import { findPaths } from "../../core/src/path_search.mjs";
import { renderPath } from "../../core/src/renderers.mjs";

export function runCli(argv, io = defaultIo()) {
  let exitCode = 0;
  const program = createProgram(io);

  try {
    program.parse(argv, { from: "user" });
  } catch (error) {
    if (typeof error.exitCode === "number") {
      exitCode = error.exitCode;
    } else {
      throw error;
    }
  }

  return exitCode;
}

export function createProgram(io = defaultIo()) {
  const program = new Command();

  program
    .name("schema-pathfinder")
    .description("Find declared PostgreSQL FK paths between tables")
    .exitOverride()
    .configureOutput({
      writeOut(value) {
        io.stdout(value);
      },
      writeErr(value) {
        io.stderr(value);
      }
    });

  program
    .command("path")
    .description("Find a declared FK path between two tables")
    .argument("<source-table>", "source table")
    .argument("<target-table>", "target table")
    .option("--format <format>", "output format", "text")
    .action((sourceTable, targetTable, options) => {
      executePathCommand(sourceTable, targetTable, options, io);
    });

  return program;
}

function executePathCommand(sourceTable, targetTable, options, io) {
  const parsedFormat = parseOutputFormat(options.format);

  if (parsedFormat.success === false) {
    io.stderr(`Unsupported format: ${options.format}\n`);
    io.stderr(`Supported formats: ${formatList()}\n`);
    throw Object.assign(new Error("Unsupported format"), { exitCode: 1 });
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

  io.stdout(`${renderPath(result.paths[0], parsedFormat.data)}\n`);
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
