//! Command-line surface of the `schema-pathfinder` binary.
//!
//! This module owns everything the terminal user can type: the command names
//! (`tables`, `schemas`, `databases`, `tree`, `fixture`, `path`, `help`,
//! `version`), the global options (`--format`, `--fixture`, `--sql`,
//! `--database-url`, `--max-links`), the legacy `--tables` style flags kept for
//! backward compatibility, and the help texts.
//!
//! Parsing is deliberately **pure**: [`parse_cli`] takes an iterator of
//! arguments and returns a [`CliOptions`] value or a [`PathfinderError`]. It
//! never reads the environment, never touches the file system and never prints,
//! which makes the whole command surface testable without spawning a process.
//!
//! ```
//! use schema_pathfinder::cli::{parse_cli, CliCommand};
//!
//! let options = parse_cli(["path", "ClothingItem", "User", "--format", "sql"]).unwrap();
//!
//! assert_eq!(
//!     options.command,
//!     CliCommand::Path {
//!         source_table: "ClothingItem".to_string(),
//!         target_table: "User".to_string(),
//!     }
//! );
//! assert_eq!(options.format_name(), "sql");
//! ```

use crate::pathfinder_core::{
    clamp_max_links, parse_format, pathfinder_error, OutputFormat, PathfinderError,
    DEFAULT_MAX_LINKS, MAX_LINKS_CEILING, SUPPORTED_FORMATS,
};
use lexopt::prelude::*;
use std::ffi::OsString;

/// Binary name, used in the help text and in the `version` output.
pub const PROGRAM_NAME: &str = "schema-pathfinder";

/// Every command name accepted on the command line, aliases included.
///
/// Used to build the "unknown command" hint and to decide whether a positional
/// argument is a help topic.
pub const COMMAND_NAMES: [&str; 9] = [
    "tables", "schemas", "databases", "dbs", "tree", "path", "fixture", "help", "version",
];

/// Commands whose output is produced by a PostgreSQL server introspection and
/// which therefore refuse fixture and SQL sources.
pub const POSTGRES_ONLY_COMMANDS: [&str; 3] = ["databases", "schemas", "tree"];

/// The command selected on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Print help, optionally scoped to one command (`help path`).
    Help { topic: Option<String> },
    /// Print the version and exit.
    Version,
    /// List the tables of the selected source.
    Tables,
    /// List the databases of a PostgreSQL server.
    Databases,
    /// List the schemas of a PostgreSQL database.
    Schemas,
    /// Print the whole server architecture tree.
    Tree,
    /// Print the selected source as a fixture JSON.
    Fixture,
    /// List every foreign-key path between two tables, simplest first.
    Path {
        /// The starting table, qualified (`schema.table`) or bare.
        source_table: String,
        /// The destination table, qualified (`schema.table`) or bare.
        target_table: String,
    },
}

impl CliCommand {
    /// Canonical name of the command, as typed by the user.
    pub fn name(&self) -> &'static str {
        match self {
            CliCommand::Help { .. } => "help",
            CliCommand::Version => "version",
            CliCommand::Tables => "tables",
            CliCommand::Databases => "databases",
            CliCommand::Schemas => "schemas",
            CliCommand::Tree => "tree",
            CliCommand::Fixture => "fixture",
            CliCommand::Path { .. } => "path",
        }
    }

    /// `true` when the command can only run against a live PostgreSQL server.
    pub fn requires_postgres(&self) -> bool {
        POSTGRES_ONLY_COMMANDS.contains(&self.name())
    }

    /// `true` when the command accepts `--format`.
    ///
    /// Only `path` renders a path, so it is the only command that honours the
    /// output format. The listing commands print one entry per line so that
    /// `grep`, `sort` and shell loops keep working.
    pub fn supports_format(&self) -> bool {
        matches!(self, CliCommand::Path { .. })
    }

    /// One-line usage summary, shown at the top of every help screen.
    pub fn usage(&self) -> String {
        match self {
            CliCommand::Help { .. } => format!("{} help [command]", PROGRAM_NAME),
            CliCommand::Version => format!("{} version", PROGRAM_NAME),
            CliCommand::Tables => format!("{} tables [options]", PROGRAM_NAME),
            CliCommand::Databases => format!("{} databases [options]", PROGRAM_NAME),
            CliCommand::Schemas => format!("{} schemas [options]", PROGRAM_NAME),
            CliCommand::Tree => format!("{} tree [options]", PROGRAM_NAME),
            CliCommand::Fixture => format!("{} fixture [options]", PROGRAM_NAME),
            CliCommand::Path { .. } => {
                format!("{} path <source-table> <target-table> [options]", PROGRAM_NAME)
            }
        }
    }
}

/// The fully resolved command-line invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct CliOptions {
    /// The command to run.
    pub command: CliCommand,
    /// The requested output format (`path` only).
    pub format: OutputFormat,
    /// Fixture JSON source given with `--fixture`.
    pub fixture: Option<String>,
    /// PostgreSQL DDL file given with `--sql`.
    pub sql: Option<String>,
    /// Connection URL given with `--database-url`.
    pub database_url: Option<String>,
    /// Maximum number of foreign-key links per listed path, already clamped to
    /// `[1, MAX_LINKS_CEILING]`.
    pub max_links: usize,
}

impl CliOptions {
    /// Canonical name of the requested output format.
    pub fn format_name(&self) -> &'static str {
        crate::pathfinder_core::format_name(self.format)
    }

    /// Describe the selected metadata source in one line, for help and errors.
    pub fn source_description(&self) -> String {
        if let Some(path) = self.fixture.as_deref() {
            return format!("fixture {}", path);
        }

        if let Some(path) = self.sql.as_deref() {
            return format!("sql {}", path);
        }

        if let Some(database_url) = self.database_url.as_deref() {
            return format!("database-url {}", database_url);
        }

        "DATABASE_URL environment variable".to_string()
    }
}

/// Parse a command line into [`CliOptions`].
///
/// `args` must not contain the program name; `main` passes
/// `std::env::args_os().skip(1)`.
///
/// # Errors
///
/// `ARGS_INVALID` for anything the parser does not accept (unknown option,
/// unknown command, wrong number of positional arguments, two legacy flags at
/// once, two explicit sources at once, unparsable `--max-links`) and
/// `UNSUPPORTED_FORMAT` for an unknown `--format` value.
pub fn parse_cli<I, A>(args: I) -> Result<CliOptions, PathfinderError>
where
    I: IntoIterator<Item = A>,
    A: Into<OsString>,
{
    let mut parser = lexopt::Parser::from_args(args);
    let mut format = OutputFormat::Text;
    let mut fixture: Option<String> = None;
    let mut sql: Option<String> = None;
    let mut database_url: Option<String> = None;
    let mut max_links = DEFAULT_MAX_LINKS;
    let mut flag_command: Option<CliCommand> = None;
    let mut help_requested = false;
    let mut version_requested = false;
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = parser.next().map_err(args_invalid)? {
        match arg {
            Long("help") | Short('h') => help_requested = true,
            Long("version") | Short('V') => version_requested = true,
            Long("tables") => flag_command = Some(set_flag_command(flag_command, "tables")?),
            Long("databases") | Long("dbs") => {
                flag_command = Some(set_flag_command(flag_command, "databases")?)
            }
            Long("schemas") => flag_command = Some(set_flag_command(flag_command, "schemas")?),
            Long("tree") => flag_command = Some(set_flag_command(flag_command, "tree")?),
            Long("format") => format = parse_format(&option_value(&mut parser, "--format")?)?,
            Long("fixture") => fixture = Some(option_value(&mut parser, "--fixture")?),
            Long("sql") => sql = Some(option_value(&mut parser, "--sql")?),
            Long("database-url") => {
                database_url = Some(option_value(&mut parser, "--database-url")?)
            }
            Long("max-links") => {
                max_links = parse_max_links(&option_value(&mut parser, "--max-links")?)?
            }
            Value(value) => positionals.push(
                value.into_string().map_err(|raw| {
                    args_invalid(format!("invalid UTF-8 in argument '{}'", raw.to_string_lossy()))
                })?,
            ),
            other => return Err(args_invalid(other.unexpected())),
        }
    }

    let command = resolve_command(
        &positionals,
        flag_command,
        help_requested,
        version_requested,
    )?;
    reject_conflicting_sources(&fixture, &sql, &database_url)?;

    Ok(CliOptions {
        command,
        format,
        fixture,
        sql,
        database_url,
        max_links,
    })
}

/// Parse the command line of the current process, skipping `argv[0]`.
///
/// Convenience wrapper over [`parse_cli`] used by `main`.
pub fn parse_cli_from_env() -> Result<CliOptions, PathfinderError> {
    parse_cli(std::env::args_os().skip(1))
}

fn option_value(parser: &mut lexopt::Parser, name: &str) -> Result<String, PathfinderError> {
    let raw = parser
        .value()
        .map_err(|error| args_invalid(format!("{} needs a value ({})", name, error)))?;

    raw.into_string()
        .map_err(|_| args_invalid(format!("invalid UTF-8 in {} value", name)))
}

fn parse_max_links(raw: &str) -> Result<usize, PathfinderError> {
    let parsed: i64 = raw.parse().map_err(|_| {
        args_invalid(format!(
            "--max-links expects a whole number between 1 and {}, got '{}'",
            MAX_LINKS_CEILING, raw
        ))
    })?;

    if parsed < 1 {
        return Err(args_invalid(format!(
            "--max-links expects a whole number between 1 and {}, got '{}'",
            MAX_LINKS_CEILING, raw
        )));
    }

    Ok(clamp_max_links(usize::try_from(parsed).unwrap_or(MAX_LINKS_CEILING)))
}

fn set_flag_command(
    current: Option<CliCommand>,
    requested: &str,
) -> Result<CliCommand, PathfinderError> {
    if current.is_some() {
        return Err(args_invalid(
            "choose only one of --tables, --databases, --schemas, or --tree",
        ));
    }

    subcommand(requested, &[])
}

fn reject_conflicting_sources(
    fixture: &Option<String>,
    sql: &Option<String>,
    database_url: &Option<String>,
) -> Result<(), PathfinderError> {
    let mut provided = Vec::new();

    if fixture.is_some() {
        provided.push("--fixture");
    }

    if sql.is_some() {
        provided.push("--sql");
    }

    if database_url.is_some() {
        provided.push("--database-url");
    }

    if provided.len() > 1 {
        return Err(args_invalid(format!(
            "choose only one metadata source, got {}",
            provided.join(" and ")
        )));
    }

    Ok(())
}

fn resolve_command(
    positionals: &[String],
    flag_command: Option<CliCommand>,
    help_requested: bool,
    version_requested: bool,
) -> Result<CliCommand, PathfinderError> {
    if help_requested {
        let topic = positionals
            .first()
            .filter(|value| COMMAND_NAMES.contains(&value.as_str()))
            .cloned();

        return Ok(CliCommand::Help { topic });
    }

    if version_requested {
        return Ok(CliCommand::Version);
    }

    match (positionals.first(), flag_command) {
        (Some(name), Some(_)) => Err(args_invalid(format!(
            "choose either the '{}' subcommand or one of the legacy flags --tables, --databases, --schemas, --tree",
            name
        ))),
        (None, Some(command)) => Ok(command),
        (None, None) => Ok(CliCommand::Help { topic: None }),
        (Some(name), None) => subcommand(name, &positionals[1..]),
    }
}

fn subcommand(name: &str, args: &[String]) -> Result<CliCommand, PathfinderError> {
    match name {
        "help" => {
            if args.len() > 1 {
                return Err(args_invalid("help takes at most one command name"));
            }

            Ok(CliCommand::Help {
                topic: args.first().cloned(),
            })
        }
        "version" => Ok(reject_extra_args(name, args, CliCommand::Version)?),
        "tables" => Ok(reject_extra_args(name, args, CliCommand::Tables)?),
        "databases" | "dbs" => Ok(reject_extra_args(name, args, CliCommand::Databases)?),
        "schemas" => Ok(reject_extra_args(name, args, CliCommand::Schemas)?),
        "tree" => Ok(reject_extra_args(name, args, CliCommand::Tree)?),
        "fixture" => Ok(reject_extra_args(name, args, CliCommand::Fixture)?),
        "path" => {
            if args.len() < 2 {
                return Err(args_invalid(
                    "path requires <source-table> and <target-table>",
                ));
            }

            if args.len() > 2 {
                return Err(args_invalid(format!(
                    "path takes exactly <source-table> and <target-table>, got {} extra argument(s)",
                    args.len() - 2
                )));
            }

            Ok(CliCommand::Path {
                source_table: args[0].clone(),
                target_table: args[1].clone(),
            })
        }
        _ => Err(args_invalid(format!(
            "unknown command '{}' (expected one of: {})",
            name,
            COMMAND_NAMES.join(", ")
        ))),
    }
}

fn reject_extra_args(
    name: &str,
    args: &[String],
    command: CliCommand,
) -> Result<CliCommand, PathfinderError> {
    if !args.is_empty() {
        return Err(args_invalid(format!(
            "{} does not accept positional arguments",
            name
        )));
    }

    Ok(command)
}

fn args_invalid<T: std::fmt::Display>(message: T) -> PathfinderError {
    pathfinder_error("ARGS_INVALID", message)
}

/// Full help screen, printed by `--help`, `help` and when no argument is given.
pub fn help_text() -> String {
    let mut lines = Vec::new();

    lines.push(format!("Usage: {} <command> [options]", PROGRAM_NAME));
    lines.push(String::new());
    lines.push("Find declared PostgreSQL FK paths between tables".to_string());
    lines.push(String::new());
    lines.push("Commands:".to_string());
    lines.push(
        "  tables                   List tables from a fixture, a SQL file, or PostgreSQL".to_string(),
    );
    lines.push("  schemas                  List PostgreSQL schemas in the selected database".to_string());
    lines.push("  databases                List PostgreSQL databases (alias: dbs)".to_string());
    lines.push("  tree                     Print all visible databases, schemas, tables, and FKs".to_string());
    lines.push("  path <source> <target>   List declared FK paths between two tables, simplest first".to_string());
    lines.push("  fixture                  Print the selected source as fixture JSON".to_string());
    lines.push("  help [command]           Display help, optionally for one command".to_string());
    lines.push("  version                  Print the version and exit".to_string());
    lines.push(String::new());
    lines.push("Options:".to_string());
    lines.push(
        "  --fixture <path>         read FK metadata from a fixture JSON file".to_string(),
    );
    lines.push(
        "  --sql <path>             read PostgreSQL DDL SQL and convert it to metadata".to_string(),
    );
    lines.push(
        "  --database-url <url>     read PostgreSQL metadata from this connection URL".to_string(),
    );
    lines.push(format!(
        "  --max-links <n>          max number of FK links per listed path (default {}, capped at {})",
        DEFAULT_MAX_LINKS, MAX_LINKS_CEILING
    ));
    lines.push("  --format <fmt>           output format for `path` (see below)".to_string());
    lines.push("  -h, --help               display help for command".to_string());
    lines.push("  -V, --version            print the version and exit".to_string());
    lines.push(String::new());
    lines.push("Legacy flags (kept for compatibility, prefer the subcommands):".to_string());
    lines.push("  --tables, --schemas, --tree, --databases, --dbs".to_string());
    lines.push(String::new());
    lines.push("Supported formats:".to_string());
    lines.push(format!("  {}", SUPPORTED_FORMATS.join(", ")));
    lines.push(String::new());
    lines.push("Environment:".to_string());
    lines.push(
        "  DATABASE_URL             PostgreSQL connection string used when no --fixture, --sql, or --database-url is given"
            .to_string(),
    );
    lines.push(String::new());
    lines.push("Exit codes:".to_string());
    lines.push("  0   success".to_string());
    lines.push("  1   runtime error (missing file, unreachable database, no path, ...)".to_string());
    lines.push("  2   usage error (bad argument, unknown command, unknown format)".to_string());
    lines.push(String::new());
    lines.push("Examples:".to_string());
    lines.push(format!(
        "  {} tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json",
        PROGRAM_NAME
    ));
    lines.push(format!(
        "  {} path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json",
        PROGRAM_NAME
    ));
    lines.push(format!(
        "  {} path ClothingItem User --max-links 3 --format sql --sql fixtures/postgres/dressshot_schema.sql",
        PROGRAM_NAME
    ));
    lines.push(format!(
        "  {} databases --database-url postgres://readonly:change-me@localhost:5432/postgres",
        PROGRAM_NAME
    ));
    lines.push(format!(
        "  {} tree --database-url postgres://readonly:change-me@localhost:5432/postgres",
        PROGRAM_NAME
    ));
    lines.push(format!(
        "  {} fixture --sql fixtures/postgres/dressshot_schema.sql",
        PROGRAM_NAME
    ));
    lines.push(format!("  {} help path", PROGRAM_NAME));

    lines.join("\n")
}

/// Help screen for a single command, printed by `help <command>`.
///
/// # Errors
///
/// `ARGS_INVALID` when `topic` is not a known command name.
pub fn command_help(topic: &str) -> Result<String, PathfinderError> {
    let command = match topic {
        "tables" => CliCommand::Tables,
        "schemas" => CliCommand::Schemas,
        "databases" | "dbs" => CliCommand::Databases,
        "tree" => CliCommand::Tree,
        "fixture" => CliCommand::Fixture,
        "path" => CliCommand::Path {
            source_table: String::new(),
            target_table: String::new(),
        },
        "version" => CliCommand::Version,
        "help" => CliCommand::Help { topic: None },
        _ => {
            return Err(args_invalid(format!(
                "unknown command '{}' (expected one of: {})",
                topic,
                COMMAND_NAMES.join(", ")
            )))
        }
    };

    Ok(command_help_text(&command))
}

/// Help screen of a resolved [`CliCommand`].
pub fn command_help_text(command: &CliCommand) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Usage: {}", command.usage()));
    lines.push(String::new());
    lines.push(command_summary(command));
    lines.push(String::new());

    if command.requires_postgres() {
        lines.push("Source: a live PostgreSQL server only.".to_string());
        lines.push(
            "  --database-url <url>     connection URL (defaults to DATABASE_URL)".to_string(),
        );
    } else {
        lines.push("Source options:".to_string());
        lines.push("  --fixture <path>         read FK metadata from a fixture JSON file".to_string());
        lines.push("  --sql <path>             read PostgreSQL DDL SQL and convert it to metadata".to_string());
        lines.push("  --database-url <url>     read PostgreSQL metadata from this connection URL".to_string());
    }

    if command.supports_format() {
        lines.push(String::new());
        lines.push("Output options:".to_string());
        lines.push(format!(
            "  --max-links <n>          max number of FK links per listed path (default {}, capped at {})",
            DEFAULT_MAX_LINKS, MAX_LINKS_CEILING
        ));
        lines.push(format!(
            "  --format <fmt>           {}",
            SUPPORTED_FORMATS.join(" | ")
        ));
    }

    lines.push(String::new());
    lines.push("Examples:".to_string());

    for example in command_examples(command) {
        lines.push(format!("  {}", example));
    }

    lines.join("\n")
}

fn command_summary(command: &CliCommand) -> String {
    match command {
        CliCommand::Help { .. } => {
            "Display help for the CLI or for one command.".to_string()
        }
        CliCommand::Version => "Print the version and exit.".to_string(),
        CliCommand::Tables => {
            "List the visible tables, one `schema.table` per line (`NO_TABLES` when empty)."
                .to_string()
        }
        CliCommand::Databases => {
            "List the PostgreSQL databases the role can see and connect to (`NO_DATABASES` when empty)."
                .to_string()
        }
        CliCommand::Schemas => {
            "List the application schemas of a PostgreSQL database (`NO_SCHEMAS` when empty).".to_string()
        }
        CliCommand::Tree => {
            "Print every visible database with its schemas, tables and outgoing foreign keys."
                .to_string()
        }
        CliCommand::Fixture => {
            "Print the selected source as fixture JSON, ready to be reused offline.".to_string()
        }
        CliCommand::Path { .. } => {
            "List every declared foreign-key path between two tables, simplest first.".to_string()
        }
    }
}

fn command_examples(command: &CliCommand) -> Vec<String> {
    let fixture = "fixtures/postgres/dressshot_seed_fk_edges.json";
    let sql_file = "fixtures/postgres/dressshot_schema.sql";
    let database_url = "postgres://readonly:change-me@localhost:5432/postgres";

    match command {
        CliCommand::Help { .. } => vec![
            format!("{} help", PROGRAM_NAME),
            format!("{} help path", PROGRAM_NAME),
        ],
        CliCommand::Version => vec![format!("{} version", PROGRAM_NAME)],
        CliCommand::Tables => vec![
            format!("{} tables --fixture {}", PROGRAM_NAME, fixture),
            format!("{} tables --sql {}", PROGRAM_NAME, sql_file),
            format!("{} tables --database-url {}", PROGRAM_NAME, database_url),
        ],
        CliCommand::Databases => vec![format!(
            "{} databases --database-url {}",
            PROGRAM_NAME, database_url
        )],
        CliCommand::Schemas => vec![format!(
            "{} schemas --database-url {}",
            PROGRAM_NAME, database_url
        )],
        CliCommand::Tree => vec![format!(
            "{} tree --database-url {}",
            PROGRAM_NAME, database_url
        )],
        CliCommand::Fixture => vec![
            format!("{} fixture --sql {}", PROGRAM_NAME, sql_file),
            format!("{} fixture --fixture {}", PROGRAM_NAME, fixture),
        ],
        CliCommand::Path { .. } => vec![
            format!(
                "{} path ClothingItem User --format equation --fixture {}",
                PROGRAM_NAME, fixture
            ),
            format!(
                "{} path ClothingItem User --max-links 3 --format sql --sql {}",
                PROGRAM_NAME, sql_file
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<CliOptions, PathfinderError> {
        parse_cli(args.iter().copied())
    }

    fn command(args: &[&str]) -> CliCommand {
        parse(args).expect("parses").command
    }

    fn error(args: &[&str]) -> PathfinderError {
        parse(args).expect_err("should not parse")
    }

    #[test]
    fn parses_the_path_command_with_its_options() {
        let options = parse(&["path", "A", "B", "--format", "sql", "--max-links", "3"]).expect("parses");

        assert_eq!(
            options.command,
            CliCommand::Path {
                source_table: "A".to_string(),
                target_table: "B".to_string()
            }
        );
        assert_eq!(options.format_name(), "sql");
        assert_eq!(options.max_links, 3);
    }

    #[test]
    fn accepts_options_before_the_subcommand() {
        let options = parse(&["--format", "mermaid", "path", "A", "B"]).expect("parses");

        assert_eq!(options.format_name(), "mermaid");
    }

    #[test]
    fn maps_every_listing_command() {
        assert_eq!(command(&["tables"]), CliCommand::Tables);
        assert_eq!(command(&["schemas"]), CliCommand::Schemas);
        assert_eq!(command(&["databases"]), CliCommand::Databases);
        assert_eq!(command(&["dbs"]), CliCommand::Databases);
        assert_eq!(command(&["tree"]), CliCommand::Tree);
        assert_eq!(command(&["fixture"]), CliCommand::Fixture);
    }

    #[test]
    fn keeps_the_legacy_flags_working() {
        assert_eq!(command(&["--tables"]), CliCommand::Tables);
        assert_eq!(command(&["--schemas"]), CliCommand::Schemas);
        assert_eq!(command(&["--databases"]), CliCommand::Databases);
        assert_eq!(command(&["--dbs"]), CliCommand::Databases);
        assert_eq!(command(&["--tree"]), CliCommand::Tree);
    }

    #[test]
    fn no_argument_prints_help() {
        assert_eq!(command(&[]), CliCommand::Help { topic: None });
    }

    #[test]
    fn help_wins_over_everything_else() {
        assert_eq!(
            command(&["path", "--help"]),
            CliCommand::Help {
                topic: Some("path".to_string())
            }
        );
        assert_eq!(
            command(&["--help"]),
            CliCommand::Help { topic: None }
        );
    }

    #[test]
    fn version_is_available_as_flag_and_subcommand() {
        assert_eq!(command(&["--version"]), CliCommand::Version);
        assert_eq!(command(&["-V"]), CliCommand::Version);
        assert_eq!(command(&["version"]), CliCommand::Version);
    }

    #[test]
    fn rejects_two_legacy_flags() {
        let error = error(&["--tables", "--tree"]);

        assert_eq!(error.code, "ARGS_INVALID");
        assert!(error.message.contains("choose only one of"));
    }

    #[test]
    fn rejects_a_subcommand_mixed_with_a_legacy_flag() {
        let error = error(&["tables", "--tree"]);

        assert_eq!(error.code, "ARGS_INVALID");
        assert!(error.message.contains("legacy flags"));
    }

    #[test]
    fn rejects_two_explicit_sources() {
        let error = error(&["tables", "--fixture", "a.json", "--sql", "b.sql"]);

        assert_eq!(error.code, "ARGS_INVALID");
        assert!(error.message.contains("choose only one metadata source"));
    }

    #[test]
    fn rejects_unknown_commands_with_the_list_of_valid_ones() {
        let error = error(&["tabels"]);

        assert_eq!(error.code, "ARGS_INVALID");
        assert!(error.message.contains("unknown command 'tabels'"));
        assert!(error.message.contains("path"));
    }

    #[test]
    fn rejects_wrong_positional_counts() {
        assert_eq!(
            error(&["tables", "extra"]).message,
            "tables does not accept positional arguments"
        );
        assert_eq!(
            error(&["path", "only-one"]).message,
            "path requires <source-table> and <target-table>"
        );
        assert!(error(&["path", "A", "B", "C"])
            .message
            .contains("takes exactly"));
    }

    #[test]
    fn rejects_a_bad_max_links_value() {
        assert_eq!(error(&["path", "A", "B", "--max-links", "abc"]).code, "ARGS_INVALID");
        assert_eq!(error(&["path", "A", "B", "--max-links", "-2"]).code, "ARGS_INVALID");
    }

    #[test]
    fn clamps_max_links_into_the_supported_range() {
        assert_eq!(parse(&["path", "A", "B", "--max-links", "1"]).unwrap().max_links, 1);
        assert_eq!(parse(&["path", "A", "B", "--max-links", "99"]).unwrap().max_links, MAX_LINKS_CEILING);
    }

    #[test]
    fn rejects_an_unknown_format() {
        let error = error(&["path", "A", "B", "--format", "yaml"]);

        assert_eq!(error.code, "UNSUPPORTED_FORMAT");
    }

    #[test]
    fn rejects_an_option_that_needs_a_value() {
        let error = error(&["--fixture"]);

        assert_eq!(error.code, "ARGS_INVALID");
        assert!(error.message.contains("--fixture needs a value"));
    }

    #[test]
    fn describes_the_selected_source() {
        assert_eq!(
            parse(&["tables", "--fixture", "a.json"]).unwrap().source_description(),
            "fixture a.json"
        );
        assert_eq!(
            parse(&["tables", "--sql", "a.sql"]).unwrap().source_description(),
            "sql a.sql"
        );
        assert_eq!(
            parse(&["tables"]).unwrap().source_description(),
            "DATABASE_URL environment variable"
        );
    }

    #[test]
    fn postgres_only_commands_are_flagged() {
        assert!(CliCommand::Databases.requires_postgres());
        assert!(CliCommand::Schemas.requires_postgres());
        assert!(CliCommand::Tree.requires_postgres());
        assert!(!CliCommand::Tables.requires_postgres());
        assert!(!CliCommand::Path {
            source_table: "A".to_string(),
            target_table: "B".to_string()
        }
        .requires_postgres());
    }

    #[test]
    fn only_path_supports_the_output_format() {
        assert!(CliCommand::Path {
            source_table: "A".to_string(),
            target_table: "B".to_string()
        }
        .supports_format());
        assert!(!CliCommand::Tables.supports_format());
    }

    #[test]
    fn help_text_documents_every_command_and_the_exit_codes() {
        let help = help_text();

        for name in ["tables", "schemas", "databases", "tree", "path", "fixture"] {
            assert!(help.contains(name), "help should document {}", name);
        }

        assert!(help.contains("--max-links <n>"));
        assert!(help.contains("Exit codes:"));
        assert!(help.contains("DATABASE_URL"));
    }

    #[test]
    fn per_command_help_is_available_for_every_documented_command() {
        for topic in ["tables", "schemas", "databases", "dbs", "tree", "fixture", "path", "version"] {
            let help = command_help(topic).expect("help renders");

            assert!(help.starts_with(&format!("Usage: {}", PROGRAM_NAME)));
        }

        assert!(command_help("nope").is_err());
    }

    #[test]
    fn path_help_documents_format_and_max_links() {
        let help = command_help("path").expect("help renders");

        assert!(help.contains("<source-table> <target-table>"));
        assert!(help.contains("--max-links <n>"));
        assert!(help.contains("--format <fmt>"));
    }

    #[test]
    fn listing_help_refuses_fixture_and_sql_sources() {
        let help = command_help("tree").expect("help renders");

        assert!(help.contains("live PostgreSQL server only"));
        assert!(!help.contains("--fixture"));
    }
}
