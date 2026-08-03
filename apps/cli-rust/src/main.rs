use lexopt::prelude::*;
use schema_pathfinder::pathfinder_core::{
    best_path, list_tables_from_edges, load_edges_from_env_or_fixture, parse_format, pathfinder_error,
    render_path, render_tables, OutputFormat, PathfinderError, SUPPORTED_FORMATS,
};

#[derive(Debug)]
struct Args {
    command: CommandMode,
    format: OutputFormat,
    fixture: Option<String>,
}

#[derive(Debug)]
enum CommandMode {
    Help,
    Tables,
    Path { source_table: String, target_table: String },
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
            let edges = load_edges_from_env_or_fixture(args.fixture.as_deref())?;
            println!("{}", render_tables(&list_tables_from_edges(&edges)));
            Ok(())
        }
        CommandMode::Path {
            ref source_table,
            ref target_table,
        } => {
            let edges = load_edges_from_env_or_fixture(args.fixture.as_deref())?;
            let path = best_path(&edges, source_table, target_table)?;
            println!("{}", render_path(&path, args.format)?);
            Ok(())
        }
    }
}

fn parse_args() -> Result<Args, PathfinderError> {
    let mut parser = lexopt::Parser::from_env();
    let mut values: Vec<String> = Vec::new();
    let mut format = OutputFormat::Text;
    let mut fixture = None;
    let mut top_level_tables = false;

    while let Some(arg) = parser.next().map_err(|error| pathfinder_error("ARGS_INVALID", error))? {
        match arg {
            Long("help") | Short('h') => {
                return Ok(Args {
                    command: CommandMode::Help,
                    format,
                    fixture,
                });
            }
            Long("tables") => {
                top_level_tables = true;
            }
            Long("format") => {
                let raw_value = parser.value().map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value.parse().map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                format = parse_format(&value)?;
            }
            Long("fixture") => {
                let raw_value = parser.value().map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                let value: String = raw_value.parse().map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                fixture = Some(value);
            }
            Value(value) => {
                let parsed: String = value.string().map_err(|error| pathfinder_error("ARGS_INVALID", error))?;
                values.push(parsed);
            }
            other => {
                return Err(pathfinder_error("ARGS_INVALID", other.unexpected()));
            }
        }
    }

    let command = command_from_values(top_level_tables, values)?;

    Ok(Args {
        command,
        format,
        fixture,
    })
}

fn command_from_values(top_level_tables: bool, values: Vec<String>) -> Result<CommandMode, PathfinderError> {
    if top_level_tables {
        return Ok(CommandMode::Tables);
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

fn print_help() {
    println!("Usage: schema-pathfinder <command> [options]");
    println!();
    println!("Find declared PostgreSQL FK paths between tables");
    println!();
    println!("Options:");
    println!("  --tables                 list available tables and exit");
    println!("  -h, --help               display help for command");
    println!();
    println!("Commands:");
    println!("  tables                   List available tables from fixture or PostgreSQL metadata");
    println!("  path <source> <target>   Find a declared FK path between two tables");
    println!();
    println!("Examples:");
    println!("  schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!("  schema-pathfinder tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!("  schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!("  schema-pathfinder path ClothingItem User --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json");
    println!();
    println!("Environment:");
    println!("  DATABASE_URL                  PostgreSQL connection string when no --fixture is provided");
    println!();
    println!("Supported formats:");
    println!("  {}", SUPPORTED_FORMATS.join(", "));
}
