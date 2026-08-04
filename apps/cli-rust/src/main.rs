use lexopt::prelude::*;
use schema_pathfinder::pathfinder_core::{
    best_path, discover_postgres_databases, discover_postgres_schema, discover_postgres_schemas,
    load_schema_from_env_or_fixture, load_schema_from_fixture, load_schema_from_sql_file,
    parse_format, pathfinder_error, render_names, render_path, render_tables, OutputFormat,
    PathfinderError, SchemaMetadata, SUPPORTED_FORMATS,
};
use std::env;

#[derive(Debug)]
struct Args {
    command: CommandMode,
    format: OutputFormat,
    fixture: Option<String>,
    sql: Option<String>,
    database_url: Option<String>,
}

#[derive(Debug)]
enum CommandMode {
    Help,
    Tables,
    Databases,
    Schemas,
    Fixture,
    Path {
        source_table: String,
        target_table: String,
    },
}

fn main() {
    let result = run();

    if let Err(error) = result {
        eprintln!("{}: {}", error.code, error.message);
        std::process::exit(1);
    }
}

fn run() -> Result<(), PathfinderError> {
    let args = parse_args()?;

    match args.command {
        CommandMode::Help => {
            print_help();
            Ok(())
        }
        CommandMode::Tables => {
            let schema = load_schema(&args)?;
            println!("{}", render_tables(&schema.tables));
            Ok(())
        }
        CommandMode::Databases => {
            let database_url = load_postgres_url(&args)?;
            let databases = discover_postgres_databases(&database_url)?;
            println!("{}", render_names(&databases, "NO_DATABASES"));
            Ok(())
        }
        CommandMode::Schemas => {
            let database_url = load_postgres_url(&args)?;
            let schemas = discover_postgres_schemas(&database_url)?;
            println!("{}", render_names(&schemas, "NO_SCHEMAS"));
            Ok(())
        }
        CommandMode::Path {
            ref source_table,
            ref target_table,
        } => {
            let schema = load_schema(&args)?;
            let path = best_path(&schema.edges, source_table, target_table)?;
            println!("{}", render_path(&path, args.format)?);
            Ok(())
        }
        CommandMode::Fixture => {
            let schema = load_schema(&args)?;
            let fixture = serde_json::to_string_pretty(&schema)
                .map_err(|error| pathfinder_error("FIXTURE_RENDER_FAILED", error))?;
            println!("{}", fixture);
            Ok(())
        }
    }
}

fn parse_args() -> Result<Args, PathfinderError> {
    let mut parser = lexopt::Parser::from_env();
    let mut values: Vec<String> = Vec::new();
    let mut format = OutputFormat::Text;
    let mut fixture = None;
    let mut sql = None;
    let mut database_url = None;
    let mut top_level_tables = false;
    let mut top_level_databases = false;
    let mut top_level_schemas = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| pathfinder_error("ARGS_INVALID", error))?
    {
        match arg {
            Long("help") | Short('h') => {
                return Ok(Args {
                    command: CommandMode::Help,
                    format,
                    fixture,
                    sql,
                    database_url: None,
                });
            }
            Long("tables") => {
                top_level_tables = true;
            }
            Long("databases") | Long("dbs") => {
                top_level_databases = true;
            }
            Long("schemas") => {
                top_level_schemas = true;
            }
            Long("format") => {
                let raw_value = parser
                    .value()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value
                    .parse()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                format = parse_format(&value)?;
            }
            Long("fixture") => {
                let raw_value = parser
                    .value()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value
                    .parse()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                fixture = Some(value);
            }
            Long("sql") => {
                let raw_value = parser
                    .value()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value
                    .parse()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                sql = Some(value);
            }
            Long("database-url") => {
                let raw_value = parser
                    .value()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value
                    .parse()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                database_url = Some(value);
            }
            Value(value) => {
                let parsed: String = value
                    .string()
                    .map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                values.push(parsed);
            }
            other => {
                return Err(pathfinder_error("ARGS_INVALID", other.unexpected()));
            }
        }
    }

    let command = command_from_values(
        top_level_tables,
        top_level_databases,
        top_level_schemas,
        values,
    )?;

    Ok(Args {
        command,
        format,
        fixture,
        sql,
        database_url,
    })
}

fn load_schema(args: &Args) -> Result<SchemaMetadata, PathfinderError> {
    if let Some(path) = args.fixture.as_deref() {
        return load_schema_from_fixture(path);
    }

    if let Some(path) = args.sql.as_deref() {
        return load_schema_from_sql_file(path);
    }

    if let Some(database_url) = args.database_url.as_deref() {
        return discover_postgres_schema(database_url);
    }

    load_schema_from_env_or_fixture(None)
}

fn load_postgres_url(args: &Args) -> Result<String, PathfinderError> {
    if let Some(path) = args.fixture.as_deref() {
        return Err(PathfinderError {
            code: "SOURCE_KIND_UNSUPPORTED",
            message: format!(
                "PostgreSQL metadata commands cannot use fixture source: {}",
                path
            ),
        });
    }

    if let Some(path) = args.sql.as_deref() {
        return Err(PathfinderError {
            code: "SOURCE_KIND_UNSUPPORTED",
            message: format!(
                "PostgreSQL metadata commands cannot use SQL source: {}",
                path
            ),
        });
    }

    if let Some(database_url) = args.database_url.as_ref() {
        return Ok(database_url.clone());
    }

    env::var("DATABASE_URL").map_err(|error| pathfinder_error("DB_CONFIG_MISSING", error))
}

fn command_from_values(
    top_level_tables: bool,
    top_level_databases: bool,
    top_level_schemas: bool,
    values: Vec<String>,
) -> Result<CommandMode, PathfinderError> {
    let top_level_count =
        count_top_level_commands(top_level_tables, top_level_databases, top_level_schemas);

    if top_level_count > 1 {
        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "choose only one of --tables, --databases, or --schemas".to_string(),
        });
    }

    if top_level_tables {
        return Ok(CommandMode::Tables);
    }

    if top_level_databases {
        return Ok(CommandMode::Databases);
    }

    if top_level_schemas {
        return Ok(CommandMode::Schemas);
    }

    if values.is_empty() {
        return Ok(CommandMode::Help);
    }

    if values[0] == "tables" {
        if values.len() == 1 {
            return Ok(CommandMode::Tables);
        }

        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "tables does not accept positional arguments".to_string(),
        });
    }

    if values[0] == "databases" || values[0] == "dbs" {
        if values.len() == 1 {
            return Ok(CommandMode::Databases);
        }

        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "databases does not accept positional arguments".to_string(),
        });
    }

    if values[0] == "schemas" {
        if values.len() == 1 {
            return Ok(CommandMode::Schemas);
        }

        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "schemas does not accept positional arguments".to_string(),
        });
    }

    if values[0] == "fixture" {
        if values.len() == 1 {
            return Ok(CommandMode::Fixture);
        }

        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "fixture does not accept positional arguments".to_string(),
        });
    }

    if values[0] == "path" {
        if values.len() == 3 {
            return Ok(CommandMode::Path {
                source_table: values[1].clone(),
                target_table: values[2].clone(),
            });
        }

        return Err(PathfinderError {
            code: "ARGS_INVALID",
            message: "path requires <source-table> and <target-table>".to_string(),
        });
    }

    Err(PathfinderError {
        code: "ARGS_INVALID",
        message: format!("unknown command '{}'", values[0]),
    })
}

fn count_top_level_commands(tables: bool, databases: bool, schemas: bool) -> usize {
    let mut count = 0;

    if tables {
        count += 1;
    }

    if databases {
        count += 1;
    }

    if schemas {
        count += 1;
    }

    count
}

fn print_help() {
    println!("Usage: schema-pathfinder <command> [options]");
    println!();
    println!("Find declared PostgreSQL FK paths between tables");
    println!();
    println!("Options:");
    println!("  --tables                 list available tables and exit");
    println!("  --databases, --dbs       list PostgreSQL databases and exit");
    println!("  --schemas                list PostgreSQL schemas and exit");
    println!("  --database-url <url>     read PostgreSQL metadata from this connection URL");
    println!("  --fixture <path>         read FK metadata from a fixture JSON file");
    println!("  --sql <path>             read PostgreSQL DDL SQL and convert it to metadata");
    println!("  -h, --help               display help for command");
    println!();
    println!("Commands:");
    println!(
        "  tables                   List available tables from fixture, SQL, or PostgreSQL metadata"
    );
    println!("  databases, dbs           List PostgreSQL databases");
    println!("  schemas                  List PostgreSQL schemas in the selected database");
    println!("  fixture                  Print generated fixture JSON from the selected source");
    println!("  path <source> <target>   Find a declared FK path between two tables");
    println!();
    println!("Examples:");
    println!(
        "  schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json"
    );
    println!("  schema-pathfinder tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!("  schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!("  schema-pathfinder path ClothingItem User --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!(
        "  schema-pathfinder --tables --database-url postgres://user:pass@localhost:5432/postgres"
    );
    println!(
        "  schema-pathfinder databases --database-url postgres://user:pass@localhost:5432/postgres"
    );
    println!(
        "  schema-pathfinder schemas --database-url postgres://user:pass@localhost:5432/postgres"
    );
    println!("  schema-pathfinder --tables --sql schema.sql");
    println!("  schema-pathfinder fixture --sql schema.sql");
    println!("  schema-pathfinder path ClothingItem User --format equation --sql schema.sql");
    println!();
    println!("Environment:");
    println!("  DATABASE_URL                  PostgreSQL connection string when no --fixture, --sql, or --database-url is provided");
    println!();
    println!("Supported formats:");
    println!("  {}", SUPPORTED_FORMATS.join(", "));
}
