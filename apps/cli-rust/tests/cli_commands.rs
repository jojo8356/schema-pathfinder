//! End-to-end tests for the CLI binary.
//!
//! These run the real compiled binary with the exact commands a user types and
//! check stdout, stderr and the exit code. They are the guard rail of the
//! stabilized command surface: `tables`, `schemas`, `databases`, `tree`,
//! `fixture`, `path`, `help` and `version`.
//!
//! The PostgreSQL-only commands (`databases`, `schemas`, `tree`) are exercised
//! on their error paths, because no live server is available in CI.

mod common;

use common::write_temp_file;
use std::process::Command;

/// The compiled CLI binary, injected by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_schema-pathfinder");

const DISCONNECTED_FIXTURE: &str = r#"{
  "edges": [
    {"constraintName":"A_bId_fkey","from":{"schema":"public","table":"A"},"fromColumn":"bId","to":{"schema":"public","table":"B"},"toColumn":"id","evidence":["declared_fk"]},
    {"constraintName":"X_yId_fkey","from":{"schema":"public","table":"X"},"fromColumn":"yId","to":{"schema":"public","table":"Y"},"toColumn":"id","evidence":["declared_fk"]}
  ]
}"#;

struct CliOutput {
    status: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> CliOutput {
    let output = Command::new(BIN)
        .args(args)
        .env_remove("DATABASE_URL")
        .output()
        .expect("the CLI binary runs");

    CliOutput {
        status: output.status.code().expect("the CLI exits with a code"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Absolute path of a file stored in `fixtures/postgres`.
fn fixture(relative: &str) -> String {
    format!(
        "{}/../../fixtures/postgres/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative
    )
}

fn seed_fixture() -> String {
    fixture("dressshot_seed_fk_edges.json")
}

fn dump_fixture() -> String {
    fixture("dressshot_schema.sql")
}

// ---- tables ---------------------------------------------------------------

#[test]
fn tables_lists_schema_qualified_tables_from_a_fixture() {
    let output = run(&["tables", "--fixture", seed_fixture().as_str()]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert_eq!(
        output.stdout.trim(),
        [
            "public.ClothingItem",
            "public.ClothingSession",
            "public.SellerProfile",
            "public.User"
        ]
        .join("\n")
    );
}

#[test]
fn tables_accepts_the_legacy_flag_spelling() {
    let output = run(&["--tables", "--fixture", seed_fixture().as_str()]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert!(output.stdout.contains("public.ClothingItem"));
}

#[test]
fn tables_reads_a_sql_dump() {
    let output = run(&["tables", "--sql", dump_fixture().as_str()]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert_eq!(output.stdout.lines().count(), 4);
    assert!(output.stdout.contains("public.ClothingItem"));
}

#[test]
fn tables_reports_a_missing_file_with_a_stable_code() {
    let output = run(&["tables", "--fixture", "/nonexistent/fixture.json"]);

    assert_eq!(output.status, 1);
    assert!(output.stderr.starts_with("FIXTURE_READ_FAILED"));
    assert!(output.stdout.is_empty());
}

// ---- path -----------------------------------------------------------------

#[test]
fn path_lists_every_route_simplest_first() {
    let output = run(&[
        "path",
        "ClothingItem",
        "User",
        "--format",
        "equation",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert!(output.stdout.starts_with("Path 1 score 156 length 3"));
    assert!(output
        .stdout
        .contains("ClothingItem.clothingSessionId = ClothingSession.id"));
    assert!(output.stdout.contains("SellerProfile.userId = User.id"));
}

#[test]
fn path_renders_a_read_only_sql_skeleton() {
    let output = run(&[
        "path",
        "ClothingItem",
        "User",
        "--format",
        "sql",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert!(output.stdout.contains("select *"));
    assert!(output.stdout.contains("limit 50;"));
}

#[test]
fn path_accepts_schema_qualified_table_names() {
    let output = run(&[
        "path",
        "public.ClothingItem",
        "public.User",
        "--format",
        "mermaid",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);
    assert!(output.stdout.contains("flowchart LR"));
}

#[test]
fn path_honours_max_links() {
    let output = run(&[
        "path",
        "ClothingItem",
        "User",
        "--max-links",
        "1",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 1);
    assert!(output.stderr.starts_with("NO_DECLARED_FK_PATH"));
}

#[test]
fn path_accepts_options_before_the_subcommand() {
    let output = run(&[
        "--format",
        "json",
        "--fixture",
        seed_fixture().as_str(),
        "path",
        "ClothingItem",
        "User",
    ]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);

    let parsed: serde_json::Value = serde_json::from_str(&output.stdout).expect("valid JSON");
    assert_eq!(parsed.as_array().expect("array of paths").len(), 1);
}

#[test]
fn path_reports_unknown_tables_and_missing_routes() {
    let unknown = run(&["path", "Nope", "User", "--fixture", seed_fixture().as_str()]);
    assert_eq!(unknown.status, 1);
    assert!(unknown.stderr.starts_with("TABLE_NOT_FOUND"));

    let disconnected = write_temp_file("disconnected.json", DISCONNECTED_FIXTURE);
    let output = run(&[
        "path",
        "A",
        "Y",
        "--fixture",
        disconnected.to_str().expect("utf-8 path"),
    ]);

    assert_eq!(output.status, 1);
    assert!(output.stderr.starts_with("NO_DECLARED_FK_PATH"));
}

// ---- fixture --------------------------------------------------------------

#[test]
fn fixture_converts_a_sql_dump_into_json() {
    let output = run(&["fixture", "--sql", dump_fixture().as_str()]);

    assert_eq!(output.status, 0, "stderr: {}", output.stderr);

    let parsed: serde_json::Value = serde_json::from_str(&output.stdout).expect("valid JSON");
    assert_eq!(parsed["edges"].as_array().expect("edges").len(), 3);
    assert_eq!(parsed["tables"].as_array().expect("tables").len(), 4);
}

// ---- PostgreSQL-only commands --------------------------------------------

#[test]
fn postgres_commands_refuse_a_fixture_source() {
    for command in ["databases", "schemas", "tree"] {
        let output = run(&[command, "--fixture", seed_fixture().as_str()]);

        assert_eq!(output.status, 1, "{} should fail", command);
        assert!(
            output.stderr.starts_with("SOURCE_KIND_UNSUPPORTED"),
            "{} stderr: {}",
            command,
            output.stderr
        );
    }
}

#[test]
fn postgres_commands_refuse_a_sql_source() {
    let output = run(&["schemas", "--sql", dump_fixture().as_str()]);

    assert_eq!(output.status, 1);
    assert!(output.stderr.starts_with("SOURCE_KIND_UNSUPPORTED"));
}

#[test]
fn postgres_commands_ask_for_a_connection_when_none_is_given() {
    for command in ["databases", "dbs", "schemas", "tree"] {
        let output = run(&[command]);

        assert_eq!(output.status, 1, "{} should fail", command);
        assert!(
            output.stderr.starts_with("DB_CONFIG_MISSING"),
            "{} stderr: {}",
            command,
            output.stderr
        );
    }
}

#[test]
fn postgres_commands_report_an_unreachable_server() {
    let output = run(&[
        "schemas",
        "--database-url",
        "postgres://nobody:nopass@127.0.0.1:1/nodb",
    ]);

    assert_eq!(output.status, 1);
    assert!(output.stderr.starts_with("DB_CONNECT_FAILED"));
}

// ---- usage errors ---------------------------------------------------------

#[test]
fn unknown_command_exits_with_the_usage_code() {
    let output = run(&["tabels"]);

    assert_eq!(output.status, 2);
    assert!(output.stderr.starts_with("ARGS_INVALID"));
    assert!(output.stderr.contains("unknown command 'tabels'"));
}

#[test]
fn wrong_arity_exits_with_the_usage_code() {
    let output = run(&["path", "ClothingItem"]);

    assert_eq!(output.status, 2);
    assert!(output
        .stderr
        .contains("path requires <source-table> and <target-table>"));
}

#[test]
fn an_unknown_format_exits_with_the_usage_code() {
    let output = run(&[
        "path",
        "ClothingItem",
        "User",
        "--format",
        "yaml",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 2);
    assert!(output.stderr.starts_with("UNSUPPORTED_FORMAT"));
}

#[test]
fn a_bad_max_links_value_exits_with_the_usage_code() {
    let output = run(&[
        "path",
        "ClothingItem",
        "User",
        "--max-links",
        "many",
        "--fixture",
        seed_fixture().as_str(),
    ]);

    assert_eq!(output.status, 2);
    assert!(output.stderr.contains("--max-links expects a whole number"));
}

#[test]
fn two_sources_at_once_exit_with_the_usage_code() {
    let output = run(&[
        "tables",
        "--fixture",
        seed_fixture().as_str(),
        "--sql",
        dump_fixture().as_str(),
    ]);

    assert_eq!(output.status, 2);
    assert!(output
        .stderr
        .contains("choose only one metadata source"));
}

// ---- help and version -----------------------------------------------------

#[test]
fn no_argument_prints_the_help() {
    let output = run(&[]);

    assert_eq!(output.status, 0);
    assert!(output.stdout.starts_with("Usage: schema-pathfinder"));
    for command in ["tables", "schemas", "databases", "tree", "path", "fixture"] {
        assert!(
            output.stdout.contains(command),
            "help should document {}",
            command
        );
    }
}

#[test]
fn help_is_available_per_command() {
    let output = run(&["help", "path"]);

    assert_eq!(output.status, 0);
    assert!(output
        .stdout
        .contains("path <source-table> <target-table>"));
    assert!(output.stdout.contains("--max-links <n>"));
}

#[test]
fn a_subcommand_help_flag_does_not_run_the_command() {
    let output = run(&["tables", "--help"]);

    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("Usage: schema-pathfinder tables"));
    assert!(!output.stdout.is_empty());
    assert!(!output.stdout.contains("public.ClothingItem"));
}

#[test]
fn version_prints_the_package_version() {
    let output = run(&["--version"]);

    assert_eq!(output.status, 0);
    assert!(output.stdout.contains(env!("CARGO_PKG_VERSION")));

    let subcommand = run(&["version"]);
    assert_eq!(subcommand.stdout, output.stdout);
}
