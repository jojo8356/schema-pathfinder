import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const cli = readFileSync("apps/cli-rust/src/cli.rs", "utf8");
const bin = readFileSync("apps/cli-rust/src/main.rs", "utf8");
const api = readFileSync("apps/cli-rust/src/bin/schema-pathfinder-api.rs", "utf8");
const core = readFileSync("apps/cli-rust/src/pathfinder_core.rs", "utf8");

describe("PostgreSQL server metadata commands", () => {
  it("exposes databases, schemas, and tree as stable subcommands", () => {
    assert.match(cli, /CliCommand::Databases/);
    assert.match(cli, /CliCommand::Schemas/);
    assert.match(cli, /CliCommand::Tree/);
    assert.match(cli, /Long\("databases"\) \| Long\("dbs"\)/);
    assert.match(cli, /Long\("schemas"\)/);
    assert.match(cli, /Long\("tree"\)/);
    assert.match(cli, /"databases" \| "dbs" =>/);
    assert.match(cli, /"schemas" =>/);
    assert.match(cli, /"tree" =>/);
    assert.match(cli, /pub const COMMAND_NAMES/);
  });

  it("runs the parsed command from a thin binary", () => {
    assert.match(bin, /cli::parse_cli_from_env\(\)/);
    assert.match(bin, /CliCommand::Databases =>/);
    assert.match(bin, /CliCommand::Schemas =>/);
    assert.match(bin, /CliCommand::Tree =>/);
    assert.match(bin, /CliCommand::Tables =>/);
    assert.match(bin, /CliCommand::Path \{/);
    assert.match(bin, /discover_postgres_architecture_tree/);
  });

  it("queries pg_database and pg_namespace through Rust core", () => {
    assert.match(core, /POSTGRES_DATABASE_SQL/);
    assert.match(core, /from pg_database/);
    assert.match(core, /POSTGRES_SCHEMA_SQL/);
    assert.match(core, /from pg_namespace/);
    assert.match(core, /discover_postgres_databases/);
    assert.match(core, /discover_postgres_schemas/);
    assert.match(core, /discover_postgres_architecture_tree/);
    assert.match(core, /database_url_for_database/);
    assert.match(core, /render_postgres_architecture_tree/);
  });

  it("renders an ASCII architecture tree", () => {
    assert.match(core, /postgres/);
    assert.match(core, /database /);
    assert.match(core, /schema /);
    assert.match(core, /table /);
    assert.match(core, /fk /);
    assert.match(core, /\|--/);
    assert.match(core, /`--/);
  });

  it("exposes API routes for databases and schemas", () => {
    assert.match(api, /\/api\/databases/);
    assert.match(api, /\/api\/schemas/);
    assert.match(api, /source_databases_response/);
    assert.match(api, /source_schemas_response/);
  });
});

describe("stable CLI surface", () => {
  it("keeps argument parsing pure so it can be unit-tested", () => {
    assert.match(cli, /pub fn parse_cli<I, A>\(args: I\) -> Result<CliOptions, PathfinderError>/);
    assert.match(cli, /lexopt::Parser::from_args\(args\)/);
    assert.doesNotMatch(cli, /std::env::var\("DATABASE_URL"\)/);
    assert.doesNotMatch(cli, /println!/);
    assert.match(cli, /#\[cfg\(test\)\]\s+mod tests \{/);
  });

  it("keeps the legacy flag spelling working", () => {
    assert.match(cli, /Long\("tables"\)/);
    assert.match(cli, /Legacy flags \(kept for compatibility/);
    assert.match(cli, /fn set_flag_command\(/);
    assert.match(cli, /choose only one of --tables, --databases, --schemas, or --tree/);
  });

  it("rejects ambiguous and malformed invocations with ARGS_INVALID", () => {
    assert.match(cli, /choose only one metadata source/);
    assert.match(cli, /does not accept positional arguments/);
    assert.match(cli, /path requires <source-table> and <target-table>/);
    assert.match(cli, /unknown command/);
    assert.match(cli, /--max-links expects a whole number/);
    assert.match(cli, /legacy flags/);
  });

  it("maps usage errors to a dedicated exit code", () => {
    assert.match(core, /pub fn is_usage_error\(&self\) -> bool/);
    assert.match(core, /USAGE_ERROR_CODES/);
    assert.match(bin, /is_usage_error\(\)/);
    assert.match(bin, /EXIT_USAGE: u8 = 2/);
  });

  it("documents every command, the exit codes and the environment", () => {
    assert.match(cli, /pub fn help_text\(\) -> String/);
    assert.match(cli, /pub fn command_help\(topic: &str\) -> Result<String, PathfinderError>/);
    assert.match(cli, /Exit codes:/);
    assert.match(cli, /DATABASE_URL/);
    assert.match(bin, /CliCommand::Help \{ topic \}/);
    assert.match(bin, /CliCommand::Version =>/);
  });
});

describe("multiple ranked paths by complexity", () => {
  it("adds a --max-links option to the CLI", () => {
    assert.match(cli, /Long\("max-links"\)/);
    assert.match(cli, /pub max_links: usize/);
    assert.match(cli, /--max-links <n>/);
    assert.match(cli, /clamp_max_links/);
  });

  it("lists every path simplest-first through the Rust core", () => {
    assert.match(core, /pub fn ranked_paths_by_complexity/);
    assert.match(core, /pub fn render_paths/);
    assert.match(core, /pub const DEFAULT_MAX_LINKS/);
    assert.match(core, /pub fn clamp_max_links/);
    // Complexity ordering: length ascending, then score descending.
    assert.match(core, /left\.length\s*\n?\s*\.cmp\(&right\.length\)/);
  });

  it("uses complexity ranking in the CLI path command", () => {
    assert.match(bin, /ranked_paths_by_complexity/);
    assert.match(bin, /render_paths/);
    assert.doesNotMatch(bin, /best_path/);
  });

  it("accepts a maxLinks parameter on the API path route", () => {
    assert.match(api, /ranked_paths_by_complexity/);
    assert.match(api, /render_paths/);
    assert.match(api, /rename = "maxLinks"/);
    assert.match(api, /DEFAULT_MAX_LINKS/);
  });
});
