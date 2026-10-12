//! `schema-pathfinder` command-line binary.
//!
//! The binary is a thin shell: argument parsing lives in [`schema_pathfinder::cli`]
//! and the business logic in [`schema_pathfinder::pathfinder_core`]. `main` only
//! dispatches the parsed [`schema_pathfinder::cli::CliCommand`], prints the
//! result on stdout and maps errors to a stable exit code.

use schema_pathfinder::cli::{self, CliCommand, CliOptions};
use schema_pathfinder::pathfinder_core::{
    discover_postgres_architecture_tree, discover_postgres_databases, discover_postgres_schema,
    discover_postgres_schemas, load_schema_from_env_or_fixture, load_schema_from_fixture,
    load_schema_from_sql_file, pathfinder_error, ranked_paths_by_complexity, render_names,
    render_paths, render_postgres_architecture_tree, render_tables, PathfinderError, SchemaMetadata,
};
use std::process::ExitCode;

/// Exit code used for usage errors (bad argument, unknown command, ...).
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let options = match cli::parse_cli_from_env() {
        Ok(options) => options,
        Err(error) => return fail(&error),
    };

    match run(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
    }
}

/// Print `CODE: message` on stderr and pick the exit code from the error code.
fn fail(error: &PathfinderError) -> ExitCode {
    eprintln!("{}", error);

    if error.is_usage_error() {
        return ExitCode::from(EXIT_USAGE);
    }

    ExitCode::FAILURE
}

fn run(options: &CliOptions) -> Result<(), PathfinderError> {
    match &options.command {
        CliCommand::Help { topic } => {
            println!("{}", help_text(topic.as_deref())?);
            Ok(())
        }
        CliCommand::Version => {
            println!("{} {}", cli::PROGRAM_NAME, env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        CliCommand::Tables => {
            let schema = load_schema(options)?;
            println!("{}", render_tables(&schema.tables));
            Ok(())
        }
        CliCommand::Databases => {
            let database_url = load_postgres_url(options)?;
            let databases = discover_postgres_databases(&database_url)?;
            println!("{}", render_names(&databases, "NO_DATABASES"));
            Ok(())
        }
        CliCommand::Schemas => {
            let database_url = load_postgres_url(options)?;
            let schemas = discover_postgres_schemas(&database_url)?;
            println!("{}", render_names(&schemas, "NO_SCHEMAS"));
            Ok(())
        }
        CliCommand::Tree => {
            let database_url = load_postgres_url(options)?;
            let tree = discover_postgres_architecture_tree(&database_url)?;
            println!("{}", render_postgres_architecture_tree(&tree));
            Ok(())
        }
        CliCommand::Fixture => {
            let schema = load_schema(options)?;
            let fixture = serde_json::to_string_pretty(&schema)
                .map_err(|error| pathfinder_error("FIXTURE_RENDER_FAILED", error))?;
            println!("{}", fixture);
            Ok(())
        }
        CliCommand::Path {
            source_table,
            target_table,
        } => {
            let schema = load_schema(options)?;
            let paths = ranked_paths_by_complexity(
                &schema.edges,
                source_table,
                target_table,
                options.max_links,
            )?;
            println!("{}", render_paths(&paths, options.format)?);
            Ok(())
        }
    }
}

fn help_text(topic: Option<&str>) -> Result<String, PathfinderError> {
    match topic {
        Some(topic) => cli::command_help(topic),
        None => Ok(cli::help_text()),
    }
}

/// Load the metadata of the source selected on the command line.
fn load_schema(options: &CliOptions) -> Result<SchemaMetadata, PathfinderError> {
    if let Some(path) = options.fixture.as_deref() {
        return load_schema_from_fixture(path);
    }

    if let Some(path) = options.sql.as_deref() {
        return load_schema_from_sql_file(path);
    }

    if let Some(database_url) = options.database_url.as_deref() {
        return discover_postgres_schema(database_url);
    }

    load_schema_from_env_or_fixture(None)
}

/// Resolve the connection URL of the PostgreSQL-only commands.
fn load_postgres_url(options: &CliOptions) -> Result<String, PathfinderError> {
    if let Some(path) = options.fixture.as_deref() {
        return Err(PathfinderError {
            code: "SOURCE_KIND_UNSUPPORTED",
            message: format!(
                "PostgreSQL metadata commands cannot use fixture source: {}",
                path
            ),
        });
    }

    if let Some(path) = options.sql.as_deref() {
        return Err(PathfinderError {
            code: "SOURCE_KIND_UNSUPPORTED",
            message: format!(
                "PostgreSQL metadata commands cannot use SQL source: {}",
                path
            ),
        });
    }

    if let Some(database_url) = options.database_url.as_ref() {
        return Ok(database_url.clone());
    }

    std::env::var("DATABASE_URL").map_err(|error| pathfinder_error("DB_CONFIG_MISSING", error))
}
