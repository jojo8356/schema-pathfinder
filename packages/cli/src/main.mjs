#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { Command } from "commander";
import { formatList, parseOutputFormat } from "@schema-pathfinder/core/output_formats";
import { findPaths } from "@schema-pathfinder/core/path_search";
import { renderPath } from "@schema-pathfinder/core/renderers";
import { fetchPathFromApi } from "./api_client.mjs";
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
    .option("--api-url <url>", "call a running schema-pathfinder API instead of local metadata loading")
    .option("--admin-token <token>", "admin token for API sync mode")
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

  const apiUrl = resolveApiUrl(options, env);

  if (apiUrl !== undefined) {
    const apiResult = await fetchPathFromApi({
      apiUrl,
      adminToken: resolveAdminToken(options, env),
      sourceTable,
      targetTable,
      format: parsedFormat.data
    });

    if (apiResult.error !== undefined) {
      io.stderr(`${apiResult.error.code}: ${apiResult.error.title}\n`);
      throw Object.assign(new Error(apiResult.error.title), { exitCode: 1 });
    }

    if (apiResult.rendered !== undefined) {
      io.stdout(`${apiResult.rendered}\n`);
      return 0;
    }

    if (apiResult.noPathReason !== undefined) {
      io.stdout(`${apiResult.noPathReason}\n`);
      return 0;
    }

    io.stderr("API_RESPONSE_INVALID: API response did not contain a rendered path\n");
    throw Object.assign(new Error("Invalid API response"), { exitCode: 1 });
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

function resolveApiUrl(options, env) {
  if (options.apiUrl !== undefined && options.apiUrl.length > 0) {
    return options.apiUrl;
  }

  if (env.SCHEMA_PATHFINDER_API_URL !== undefined && env.SCHEMA_PATHFINDER_API_URL.length > 0) {
    return env.SCHEMA_PATHFINDER_API_URL;
  }

  return undefined;
}

function resolveAdminToken(options, env) {
  if (options.adminToken !== undefined && options.adminToken.length > 0) {
    return options.adminToken;
  }

  if (env.SCHEMA_PATHFINDER_ADMIN_TOKEN !== undefined && env.SCHEMA_PATHFINDER_ADMIN_TOKEN.length > 0) {
    return env.SCHEMA_PATHFINDER_ADMIN_TOKEN;
  }

  return undefined;
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
