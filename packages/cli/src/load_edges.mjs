import { readFileSync } from "node:fs";
import pg from "pg";
import { discoverPostgresForeignKeys } from "@schema-pathfinder/core/postgres_fk_metadata";

export async function loadEdges(options, env = process.env) {
  if (options.fixture !== undefined) {
    const fixture = JSON.parse(readFileSync(options.fixture, "utf8"));
    return fixture.edges;
  }

  if (env.DATABASE_URL === undefined || env.DATABASE_URL.length === 0) {
    throw createCliError(
      "DB_CONFIG_MISSING",
      "DATABASE_URL is required unless --fixture is provided",
      1
    );
  }

  const pool = new pg.Pool({
    connectionString: env.DATABASE_URL
  });

  try {
    return await discoverPostgresForeignKeys(pool, ["public"]);
  } finally {
    await pool.end();
  }
}

export function createCliError(code, message, exitCode) {
  return Object.assign(new Error(message), {
    code,
    exitCode
  });
}
