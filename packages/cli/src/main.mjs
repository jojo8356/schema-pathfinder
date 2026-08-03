#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { Command } from "commander";
import { formatList, parseOutputFormat } from "../../core/src/output_formats.mjs";
import { findPaths } from "../../core/src/path_search.mjs";
import { renderPath } from "../../core/src/renderers.mjs";
import { loadEdges } from "./load_edges.mjs";

export async function runCli(argv, io = defaultIo(), env = process.env) {
  let exitCode = 0;
  const program = createProgram(io, env);

  try {
    await program.parseAsync(argv, { from: "user" });
  } catch (error) {
    if (typeof error.exitCode === "number") {
      exitCode = error.exitCode;
      if (error.code !== undefined && error.message.length > 0) {
        io.stderr(`${error.code}: ${error.message}\n`);
      }
    } else {
      throw error;
    }
  }

  return exitCode;
}

export function createProgram(io = defaultIo(), env = process.env) {
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
    .option("--fixture <path>", "load FK metadata from a fixture file")
    .action(async (sourceTable, targetTable, options) => {
      await executePathCommand(sourceTable, targetTable, options, io, env);
    });

  return program;
}

async function executePathCommand(sourceTable, targetTable, options, io, env) {
  const parsedFormat = parseOutputFormat(options.format);

  if (parsedFormat.success === false) {
    io.stderr(`Unsupported format: ${options.format}\n`);
    io.stderr(`Supported formats: ${formatList()}\n`);
    throw Object.assign(new Error("Unsupported format"), { exitCode: 1 });
  }

  const edges = await loadEdges(options, env);
  const result = findPaths({
    edges,
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
  const exitCode = await runCli(process.argv.slice(2));
  process.exit(exitCode);
}
