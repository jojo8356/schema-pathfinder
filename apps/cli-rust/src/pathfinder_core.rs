pub use crate::postgres_sql_metadata::load_schema_from_sql;
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;

pub const SUPPORTED_FORMATS: [&str; 5] = ["text", "equation", "json", "sql", "mermaid"];

/// Default maximum number of foreign-key links (edges) a listed path may use.
/// Beyond five hops the required data entry usually becomes impractical, so the
/// UIs default to this value while still letting the user raise or lower it.
pub const DEFAULT_MAX_LINKS: usize = 5;

/// Hard ceiling on how many links a path may ever use, regardless of user input.
/// It keeps path enumeration bounded on very densely linked schemas.
pub const MAX_LINKS_CEILING: usize = 8;

/// Safety cap on the total number of ranked paths returned to a caller so a
/// pathological schema cannot flood the UI or terminal.
pub const MAX_RANKED_PATHS: usize = 50;
const POSTGRES_TABLE_SQL: &str = r#"
select
  table_ns.nspname as schema_name,
  table_record.relname as table_name
from pg_class table_record
join pg_namespace table_ns on table_ns.oid = table_record.relnamespace
where table_record.relkind in ('r', 'p')
  and table_ns.nspname = any($1)
order by table_ns.nspname, table_record.relname
"#;

const POSTGRES_DATABASE_SQL: &str = r#"
select datname
from pg_database
where datistemplate = false
  and datallowconn = true
order by datname
"#;

const POSTGRES_SCHEMA_SQL: &str = r#"
select nspname
from pg_namespace
where nspname <> 'information_schema'
  and nspname not like 'pg_%'
order by nspname
"#;

const POSTGRES_FK_SQL: &str = r#"
select
  source_ns.nspname as source_schema,
  source_table.relname as source_table,
  source_attr.attname as source_column,
  target_ns.nspname as target_schema,
  target_table.relname as target_table,
  target_attr.attname as target_column,
  constraint_record.conname as constraint_name
from pg_constraint constraint_record
join pg_class source_table on source_table.oid = constraint_record.conrelid
join pg_namespace source_ns on source_ns.oid = source_table.relnamespace
join pg_class target_table on target_table.oid = constraint_record.confrelid
join pg_namespace target_ns on target_ns.oid = target_table.relnamespace
join unnest(constraint_record.conkey) with ordinality source_key(attnum, position) on true
join unnest(constraint_record.confkey) with ordinality target_key(attnum, position)
  on source_key.position = target_key.position
join pg_attribute source_attr
  on source_attr.attrelid = source_table.oid and source_attr.attnum = source_key.attnum
join pg_attribute target_attr
  on target_attr.attrelid = target_table.oid and target_attr.attnum = target_key.attnum
where constraint_record.contype = 'f'
  and source_ns.nspname = any($1)
order by source_ns.nspname, source_table.relname, constraint_record.conname, source_key.position
"#;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TableIdentifier {
    pub schema: String,
    pub table: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub tables: Vec<TableIdentifier>,
    pub edges: Vec<ForeignKeyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresArchitectureTree {
    pub databases: Vec<DatabaseArchitecture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseArchitecture {
    pub name: String,
    pub schemas: Vec<SchemaArchitecture>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaArchitecture {
    pub name: String,
    pub tables: Vec<TableArchitecture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableArchitecture {
    pub name: String,
    pub outgoing: Vec<ForeignKeyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyEdge {
    #[serde(rename = "constraintName")]
    pub constraint_name: String,
    pub from: TableIdentifier,
    #[serde(rename = "fromColumn")]
    pub from_column: String,
    pub to: TableIdentifier,
    #[serde(rename = "toColumn")]
    pub to_column: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoredPath {
    pub source: TableIdentifier,
    pub target: TableIdentifier,
    pub score: i32,
    pub length: usize,
    pub evidence: Vec<String>,
    pub edges: Vec<ForeignKeyEdge>,
    #[serde(rename = "scoreContributions")]
    pub score_contributions: Vec<ScoreContribution>,
}

#[derive(Debug, Serialize)]
pub struct ScoreContribution {
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    #[serde(default)]
    tables: Vec<TableIdentifier>,
    edges: Vec<ForeignKeyEdge>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Equation,
    Json,
    Sql,
    Mermaid,
}

#[derive(Debug)]
pub struct PathfinderError {
    pub code: &'static str,
    pub message: String,
}

pub fn parse_format(value: &str) -> Result<OutputFormat, PathfinderError> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "equation" => Ok(OutputFormat::Equation),
        "json" => Ok(OutputFormat::Json),
        "sql" => Ok(OutputFormat::Sql),
        "mermaid" => Ok(OutputFormat::Mermaid),
        _ => Err(PathfinderError {
            code: "UNSUPPORTED_FORMAT",
            message: format!(
                "Unsupported format: {}. Supported formats: {}",
                value,
                SUPPORTED_FORMATS.join(", ")
            ),
        }),
    }
}

pub fn format_name(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Text => "text",
        OutputFormat::Equation => "equation",
        OutputFormat::Json => "json",
        OutputFormat::Sql => "sql",
        OutputFormat::Mermaid => "mermaid",
    }
}

pub fn load_schema_from_fixture(path: &str) -> Result<SchemaMetadata, PathfinderError> {
    let content =
        fs::read_to_string(path).map_err(|error| pathfinder_error("FIXTURE_READ_FAILED", error))?;
    let fixture: Fixture = serde_json::from_str(&content)
        .map_err(|error| pathfinder_error("FIXTURE_JSON_INVALID", error))?;
    let mut tables = fixture.tables;

    if tables.is_empty() {
        tables = list_tables_from_edges(&fixture.edges);
    }

    Ok(SchemaMetadata {
        tables,
        edges: fixture.edges,
    })
}

pub fn load_edges_from_fixture(path: &str) -> Result<Vec<ForeignKeyEdge>, PathfinderError> {
    Ok(load_schema_from_fixture(path)?.edges)
}

pub fn load_schema_from_sql_file(path: &str) -> Result<SchemaMetadata, PathfinderError> {
    let content =
        fs::read_to_string(path).map_err(|error| pathfinder_error("SQL_READ_FAILED", error))?;
    load_schema_from_sql(&content)
}

pub fn load_edges_from_sql_file(path: &str) -> Result<Vec<ForeignKeyEdge>, PathfinderError> {
    Ok(load_schema_from_sql_file(path)?.edges)
}

pub fn load_schema_from_env_or_fixture(
    fixture: Option<&str>,
) -> Result<SchemaMetadata, PathfinderError> {
    if let Some(path) = fixture {
        return load_schema_from_fixture(path);
    }

    let database_url =
        env::var("DATABASE_URL").map_err(|error| pathfinder_error("DB_CONFIG_MISSING", error))?;
    discover_postgres_schema(&database_url)
}

pub fn load_edges_from_env_or_fixture(
    fixture: Option<&str>,
) -> Result<Vec<ForeignKeyEdge>, PathfinderError> {
    Ok(load_schema_from_env_or_fixture(fixture)?.edges)
}

pub fn discover_postgres_databases(database_url: &str) -> Result<Vec<String>, PathfinderError> {
    let mut client = Client::connect(database_url, NoTls)
        .map_err(|error| pathfinder_error("DB_CONNECT_FAILED", error))?;
    let rows = client
        .query(POSTGRES_DATABASE_SQL, &[])
        .map_err(|error| pathfinder_error("DB_METADATA_FAILED", error))?;
    let mut databases = Vec::new();

    for row in rows {
        databases.push(row.get("datname"));
    }

    Ok(databases)
}

pub fn discover_postgres_schemas(database_url: &str) -> Result<Vec<String>, PathfinderError> {
    let mut client = Client::connect(database_url, NoTls)
        .map_err(|error| pathfinder_error("DB_CONNECT_FAILED", error))?;
    let rows = client
        .query(POSTGRES_SCHEMA_SQL, &[])
        .map_err(|error| pathfinder_error("DB_METADATA_FAILED", error))?;
    let mut schemas = Vec::new();

    for row in rows {
        schemas.push(row.get("nspname"));
    }

    Ok(schemas)
}

pub fn discover_postgres_schema(database_url: &str) -> Result<SchemaMetadata, PathfinderError> {
    // Do not assume that application tables live in `public`. PostgreSQL
    // installations may use a dedicated schema (or several schemas), which
    // is common for existing applications such as OpenConcerto.
    let schemas = discover_postgres_schemas(database_url)?;
    discover_postgres_schema_for_schemas(database_url, &schemas)
}

pub fn discover_postgres_schema_for_schemas(
    database_url: &str,
    schemas: &[String],
) -> Result<SchemaMetadata, PathfinderError> {
    let mut client = Client::connect(database_url, NoTls)
        .map_err(|error| pathfinder_error("DB_CONNECT_FAILED", error))?;
    let table_rows = client
        .query(POSTGRES_TABLE_SQL, &[&schemas])
        .map_err(|error| pathfinder_error("DB_METADATA_FAILED", error))?;
    let rows = client
        .query(POSTGRES_FK_SQL, &[&schemas])
        .map_err(|error| pathfinder_error("DB_METADATA_FAILED", error))?;
    let mut tables = Vec::new();
    let mut edges = Vec::new();

    for row in table_rows {
        tables.push(TableIdentifier {
            schema: row.get("schema_name"),
            table: row.get("table_name"),
        });
    }

    for row in rows {
        edges.push(ForeignKeyEdge {
            constraint_name: row.get("constraint_name"),
            from: TableIdentifier {
                schema: row.get("source_schema"),
                table: row.get("source_table"),
            },
            from_column: row.get("source_column"),
            to: TableIdentifier {
                schema: row.get("target_schema"),
                table: row.get("target_table"),
            },
            to_column: row.get("target_column"),
            evidence: vec!["declared_fk".to_string()],
        });
    }

    Ok(SchemaMetadata { tables, edges })
}

pub fn discover_postgres_architecture_tree(
    database_url: &str,
) -> Result<PostgresArchitectureTree, PathfinderError> {
    let database_names = discover_postgres_databases(database_url)?;
    let mut databases = Vec::new();

    for database_name in database_names {
        let database_url = database_url_for_database(database_url, &database_name);
        let database = match discover_database_architecture(&database_url, &database_name) {
            Ok(value) => value,
            Err(error) => DatabaseArchitecture {
                name: database_name,
                schemas: Vec::new(),
                error: Some(format!("{}: {}", error.code, error.message)),
            },
        };

        databases.push(database);
    }

    Ok(PostgresArchitectureTree { databases })
}

fn discover_database_architecture(
    database_url: &str,
    database_name: &str,
) -> Result<DatabaseArchitecture, PathfinderError> {
    let schema_names = discover_postgres_schemas(database_url)?;
    let schema = discover_postgres_schema_for_schemas(database_url, &schema_names)?;
    let mut schemas = Vec::new();

    for schema_name in schema_names {
        let mut tables = Vec::new();

        for table in schema
            .tables
            .iter()
            .filter(|table| table.schema == schema_name)
        {
            let outgoing = schema
                .edges
                .iter()
                .filter(|edge| edge.from == *table)
                .cloned()
                .collect();

            tables.push(TableArchitecture {
                name: table.table.clone(),
                outgoing,
            });
        }

        schemas.push(SchemaArchitecture {
            name: schema_name,
            tables,
        });
    }

    Ok(DatabaseArchitecture {
        name: database_name.to_string(),
        schemas,
        error: None,
    })
}

pub fn database_url_for_database(database_url: &str, database_name: &str) -> String {
    let Some(scheme_index) = database_url.find("://") else {
        return database_url.to_string();
    };
    let authority_start = scheme_index + 3;
    let authority_and_path = &database_url[authority_start..];
    let Some(path_offset) = authority_and_path.find('/') else {
        return format!("{}/{}", database_url, database_name);
    };
    let path_start = authority_start + path_offset;
    let prefix = &database_url[..path_start + 1];
    let path_and_query = &database_url[path_start + 1..];
    let query_start = path_and_query.find('?');
    let query = match query_start {
        Some(index) => &path_and_query[index..],
        None => "",
    };

    format!("{}{}{}", prefix, database_name, query)
}

pub fn render_postgres_architecture_tree(tree: &PostgresArchitectureTree) -> String {
    if tree.databases.is_empty() {
        return "NO_DATABASES".to_string();
    }

    let mut lines = Vec::new();
    lines.push("postgres".to_string());

    for database_index in 0..tree.databases.len() {
        let database = &tree.databases[database_index];
        let database_last = database_index + 1 == tree.databases.len();
        let database_branch = tree_branch(database_last);
        lines.push(format!("{} database {}", database_branch, database.name));
        let database_prefix = tree_prefix(database_last);

        if let Some(error) = database.error.as_ref() {
            lines.push(format!(
                "{}{} error {}",
                database_prefix,
                tree_branch(true),
                error
            ));
            continue;
        }

        if database.schemas.is_empty() {
            lines.push(format!(
                "{}{} NO_SCHEMAS",
                database_prefix,
                tree_branch(true)
            ));
            continue;
        }

        for schema_index in 0..database.schemas.len() {
            let schema = &database.schemas[schema_index];
            let schema_last = schema_index + 1 == database.schemas.len();
            lines.push(format!(
                "{}{} schema {}",
                database_prefix,
                tree_branch(schema_last),
                schema.name
            ));
            let schema_prefix = format!("{}{}", database_prefix, tree_prefix(schema_last));

            if schema.tables.is_empty() {
                lines.push(format!("{}{} NO_TABLES", schema_prefix, tree_branch(true)));
                continue;
            }

            for table_index in 0..schema.tables.len() {
                let table = &schema.tables[table_index];
                let table_last = table_index + 1 == schema.tables.len();
                lines.push(format!(
                    "{}{} table {}",
                    schema_prefix,
                    tree_branch(table_last),
                    table.name
                ));
                let table_prefix = format!("{}{}", schema_prefix, tree_prefix(table_last));

                for edge_index in 0..table.outgoing.len() {
                    let edge = &table.outgoing[edge_index];
                    let edge_last = edge_index + 1 == table.outgoing.len();
                    lines.push(format!(
                        "{}{} fk {} -> {}.{}.{} ({})",
                        table_prefix,
                        tree_branch(edge_last),
                        edge.from_column,
                        edge.to.schema,
                        edge.to.table,
                        edge.to_column,
                        edge.constraint_name
                    ));
                }
            }
        }
    }

    lines.join("\n")
}

fn tree_branch(last: bool) -> &'static str {
    if last {
        return "`--";
    }

    "|--"
}

fn tree_prefix(last: bool) -> &'static str {
    if last {
        return "   ";
    }

    "|  "
}

pub fn discover_postgres_foreign_keys(
    database_url: &str,
) -> Result<Vec<ForeignKeyEdge>, PathfinderError> {
    Ok(discover_postgres_schema(database_url)?.edges)
}

pub fn best_path(
    edges: &[ForeignKeyEdge],
    source_table: &str,
    target_table: &str,
) -> Result<ScoredPath, PathfinderError> {
    let tables = list_tables_from_edges(edges);
    let source = resolve_table(&tables, source_table).ok_or_else(|| PathfinderError {
        code: "TABLE_NOT_FOUND",
        message: format!("Source table not found: {}", source_table),
    })?;
    let target = resolve_table(&tables, target_table).ok_or_else(|| PathfinderError {
        code: "TABLE_NOT_FOUND",
        message: format!("Target table not found: {}", target_table),
    })?;
    let paths = find_paths(edges, &source, &target, 6);

    if paths.is_empty() {
        return Err(PathfinderError {
            code: "NO_DECLARED_FK_PATH",
            message: "No declared foreign-key path found".to_string(),
        });
    }

    Ok(paths.into_iter().next().expect("paths is not empty"))
}

pub fn list_tables_from_edges(edges: &[ForeignKeyEdge]) -> Vec<TableIdentifier> {
    let mut tables: BTreeMap<String, TableIdentifier> = BTreeMap::new();

    for edge in edges {
        tables.insert(table_key(&edge.from), edge.from.clone());
        tables.insert(table_key(&edge.to), edge.to.clone());
    }

    tables.into_values().collect()
}

pub fn render_names(values: &[String], empty_label: &str) -> String {
    if values.is_empty() {
        return empty_label.to_string();
    }

    values.join("\n")
}

pub fn render_tables(tables: &[TableIdentifier]) -> String {
    if tables.is_empty() {
        return "NO_TABLES".to_string();
    }

    tables
        .iter()
        .map(table_key)
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn render_path(path: &ScoredPath, format: OutputFormat) -> Result<String, PathfinderError> {
    match format {
        OutputFormat::Text => Ok(render_text(path, 0)),
        OutputFormat::Equation => Ok(render_equation(path)),
        OutputFormat::Json => serde_json::to_string_pretty(path)
            .map_err(|error| pathfinder_error("JSON_RENDER_FAILED", error)),
        OutputFormat::Sql => Ok(render_sql(path)),
        OutputFormat::Mermaid => Ok(render_mermaid(path)),
    }
}

/// Render several ranked paths as a single block, in the order they are given
/// (shortest/simplest first). Every path is numbered and, for the non-`text`
/// formats, prefixed with a short header so the reader can tell them apart.
pub fn render_paths(paths: &[ScoredPath], format: OutputFormat) -> Result<String, PathfinderError> {
    if paths.is_empty() {
        return Ok(String::new());
    }

    if let OutputFormat::Json = format {
        return serde_json::to_string_pretty(&paths)
            .map_err(|error| pathfinder_error("JSON_RENDER_FAILED", error));
    }

    let mut blocks = Vec::with_capacity(paths.len());

    for (index, path) in paths.iter().enumerate() {
        match format {
            OutputFormat::Text => blocks.push(render_text(path, index)),
            OutputFormat::Equation => blocks.push(format!(
                "{}\n{}",
                path_header(path, index),
                render_equation(path)
            )),
            OutputFormat::Sql => {
                blocks.push(format!("{}\n{}", path_header(path, index), render_sql(path)))
            }
            OutputFormat::Mermaid => blocks.push(format!(
                "{}\n{}",
                path_header(path, index),
                render_mermaid(path)
            )),
            OutputFormat::Json => unreachable!("json handled above"),
        }
    }

    Ok(blocks.join("\n\n"))
}

fn path_header(path: &ScoredPath, index: usize) -> String {
    let mut table_path = vec![path.source.table.clone()];

    for edge in &path.edges {
        table_path.push(edge.to.table.clone());
    }

    format!(
        "Path {} score {} length {}: {}",
        index + 1,
        path.score,
        path.length,
        table_path.join(" -> ")
    )
}

fn collect_paths(
    edges: &[ForeignKeyEdge],
    source: &TableIdentifier,
    target: &TableIdentifier,
    max_depth: usize,
) -> Vec<ScoredPath> {
    let adjacency = build_adjacency(edges);
    let mut queue: VecDeque<Vec<ForeignKeyEdge>> = VecDeque::new();
    let mut results = Vec::new();

    if let Some(start_edges) = adjacency.get(&table_key(source)) {
        for edge in start_edges {
            queue.push_back(vec![edge.clone()]);
        }
    }

    while let Some(path_edges) = queue.pop_front() {
        let last_edge = path_edges.last().expect("path has one edge");

        if last_edge.to == *target {
            results.push(score_path(source, target, path_edges));
            continue;
        }

        if path_edges.len() >= max_depth {
            continue;
        }

        if let Some(next_edges) = adjacency.get(&table_key(&last_edge.to)) {
            for edge in next_edges {
                if contains_table(&path_edges, &edge.to) {
                    continue;
                }

                let mut next_path = path_edges.clone();
                next_path.push(edge.clone());
                queue.push_back(next_path);
            }
        }
    }

    results
}

/// Compare two edge lists by their constraint names so paths of equal length and
/// score keep a stable, predictable order across runs.
fn edge_constraint_signature(path: &ScoredPath) -> String {
    path.edges
        .iter()
        .map(|edge| edge.constraint_name.clone())
        .collect::<Vec<String>>()
        .join("|")
}

fn find_paths(
    edges: &[ForeignKeyEdge],
    source: &TableIdentifier,
    target: &TableIdentifier,
    max_depth: usize,
) -> Vec<ScoredPath> {
    let mut results = collect_paths(edges, source, target, max_depth);

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.length.cmp(&right.length))
    });
    results.truncate(3);
    results
}

/// Return every declared foreign-key path between two tables, ordered by
/// increasing complexity: shortest paths (fewest links) first, and, within the
/// same length, the higher-scoring path first. This is what the UIs use to show
/// several join options so the user can pick one that avoids tables that are not
/// yet populated at the moment of data entry.
pub fn ranked_paths_by_complexity(
    edges: &[ForeignKeyEdge],
    source_table: &str,
    target_table: &str,
    max_links: usize,
) -> Result<Vec<ScoredPath>, PathfinderError> {
    let tables = list_tables_from_edges(edges);
    let source = resolve_table(&tables, source_table).ok_or_else(|| PathfinderError {
        code: "TABLE_NOT_FOUND",
        message: format!("Source table not found: {}", source_table),
    })?;
    let target = resolve_table(&tables, target_table).ok_or_else(|| PathfinderError {
        code: "TABLE_NOT_FOUND",
        message: format!("Target table not found: {}", target_table),
    })?;

    let depth = clamp_max_links(max_links);
    let mut results = collect_paths(edges, &source, &target, depth);

    if results.is_empty() {
        return Err(PathfinderError {
            code: "NO_DECLARED_FK_PATH",
            message: "No declared foreign-key path found".to_string(),
        });
    }

    results.sort_by(|left, right| {
        left.length
            .cmp(&right.length)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| edge_constraint_signature(left).cmp(&edge_constraint_signature(right)))
    });
    results.truncate(MAX_RANKED_PATHS);
    Ok(results)
}

/// Normalise a requested maximum link count into the supported range.
pub fn clamp_max_links(max_links: usize) -> usize {
    max_links.clamp(1, MAX_LINKS_CEILING)
}

fn build_adjacency(edges: &[ForeignKeyEdge]) -> BTreeMap<String, Vec<ForeignKeyEdge>> {
    let mut adjacency: BTreeMap<String, Vec<ForeignKeyEdge>> = BTreeMap::new();

    for edge in edges {
        adjacency
            .entry(table_key(&edge.from))
            .or_default()
            .push(edge.clone());
    }

    adjacency
}

fn contains_table(edges: &[ForeignKeyEdge], table: &TableIdentifier) -> bool {
    for edge in edges {
        if edge.from == *table || edge.to == *table {
            return true;
        }
    }

    false
}

fn score_path(
    source: &TableIdentifier,
    target: &TableIdentifier,
    edges: Vec<ForeignKeyEdge>,
) -> ScoredPath {
    let mut score = 0;
    let mut contributions = Vec::new();
    let mut evidence = BTreeSet::new();

    for edge in &edges {
        for item in &edge.evidence {
            evidence.insert(item.clone());
        }

        score += 60;
    }

    contributions.push(ScoreContribution {
        label: "declared_fk".to_string(),
        value: 60 * i32::try_from(edges.len()).unwrap_or(0),
    });

    let hop_penalty = -8 * i32::try_from(edges.len()).unwrap_or(0);
    score += hop_penalty;
    contributions.push(ScoreContribution {
        label: "extra_hop".to_string(),
        value: hop_penalty,
    });

    let mut penalized_technical_tables = BTreeSet::new();
    let mut penalized_auth_session_tables = BTreeSet::new();

    for edge in &edges {
        for table in [&edge.from.table, &edge.to.table] {
            if is_technical_table(table) && penalized_technical_tables.insert(table.clone()) {
                score -= 25;
                contributions.push(ScoreContribution {
                    label: "technical_table".to_string(),
                    value: -25,
                });
            }

            if is_auth_session_table(table) && penalized_auth_session_tables.insert(table.clone()) {
                score -= 15;
                contributions.push(ScoreContribution {
                    label: "auth_session_table".to_string(),
                    value: -15,
                });
            }
        }
    }

    ScoredPath {
        source: source.clone(),
        target: target.clone(),
        score,
        length: edges.len(),
        evidence: evidence.into_iter().collect(),
        edges,
        score_contributions: contributions,
    }
}

fn resolve_table(tables: &[TableIdentifier], table_name: &str) -> Option<TableIdentifier> {
    for table in tables {
        if table_key(table) == table_name || table.table == table_name {
            return Some(table.clone());
        }
    }

    None
}

fn table_key(table: &TableIdentifier) -> String {
    format!("{}.{}", table.schema, table.table)
}

fn render_text(path: &ScoredPath, index: usize) -> String {
    let mut table_path = vec![path.source.table.clone()];

    for edge in &path.edges {
        table_path.push(edge.to.table.clone());
    }

    let mut lines = vec![
        format!(
            "Path {} score {} {} length {}",
            index + 1,
            path.score,
            path.evidence.join(","),
            path.length
        ),
        table_path.join(" -> "),
        String::new(),
        "Edges:".to_string(),
    ];

    for (index, edge) in path.edges.iter().enumerate() {
        lines.push(format!(
            "{}. {}.{} -> {}.{}",
            index + 1,
            edge.from.table,
            edge.from_column,
            edge.to.table,
            edge.to_column
        ));
    }

    lines.join("\n")
}

fn render_equation(path: &ScoredPath) -> String {
    path.edges
        .iter()
        .map(|edge| {
            format!(
                "{}.{} = {}.{}",
                edge.from.table, edge.from_column, edge.to.table, edge.to_column
            )
        })
        .collect::<Vec<String>>()
        .join("\n-> ")
}

fn render_sql(path: &ScoredPath) -> String {
    if path.edges.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        "select *".to_string(),
        format!("from {} t0", quote_identifier(&path.source.table)),
    ];

    for (index, edge) in path.edges.iter().enumerate() {
        lines.push(format!(
            "join {} t{} on t{}.{} = t{}.{}",
            quote_identifier(&edge.to.table),
            index + 1,
            index,
            quote_identifier(&edge.from_column),
            index + 1,
            quote_identifier(&edge.to_column)
        ));
    }

    lines.push("limit 50;".to_string());
    lines.join("\n")
}

fn render_mermaid(path: &ScoredPath) -> String {
    let mut lines = vec!["flowchart LR".to_string()];

    for edge in &path.edges {
        lines.push(format!("  {} --> {}", edge.from.table, edge.to.table));
    }

    lines.join("\n")
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn is_technical_table(table: &str) -> bool {
    table == "_prisma_migrations"
}

fn is_auth_session_table(table: &str) -> bool {
    table == "Session" || table == "Account" || table == "Verification"
}

pub fn pathfinder_error<T: Display>(code: &'static str, error: T) -> PathfinderError {
    PathfinderError {
        code,
        message: error.to_string(),
    }
}

impl Display for PathfinderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for PathfinderError {}

#[cfg(test)]
mod tests {
    use super::*;

    const DRESSSHOT_SQL: &str = r#"
        create table public."User" (
            id text primary key
        );

        create table public."SellerProfile" (
            id text primary key,
            "userId" text constraint "SellerProfile_userId_fkey" references public."User"(id)
        );

        create table public."ClothingSession" (
            id text primary key,
            "sellerProfileId" text not null,
            constraint "ClothingSession_sellerProfileId_fkey"
                foreign key ("sellerProfileId") references public."SellerProfile"(id)
        );

        create table public."ClothingItem" (
            id text primary key,
            "clothingSessionId" text not null
        );

        alter table only public."ClothingItem"
            add constraint "ClothingItem_clothingSessionId_fkey"
            foreign key ("clothingSessionId") references public."ClothingSession"(id);
    "#;

    #[test]
    fn extracts_tables_and_foreign_keys_from_postgres_sql() {
        let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");

        assert_eq!(schema.tables.len(), 4);
        assert_eq!(schema.edges.len(), 3);
        assert!(schema
            .tables
            .iter()
            .any(|table| table.schema == "public" && table.table == "ClothingItem"));
        assert!(schema.edges.iter().any(|edge| {
            edge.constraint_name == "ClothingItem_clothingSessionId_fkey"
                && edge.from.table == "ClothingItem"
                && edge.from_column == "clothingSessionId"
                && edge.to.table == "ClothingSession"
                && edge.to_column == "id"
        }));
    }

    #[test]
    fn sql_schema_can_drive_normal_path_rendering() {
        let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
        let path = best_path(&schema.edges, "ClothingItem", "User").expect("path exists");
        let rendered = render_path(&path, OutputFormat::Equation).expect("path renders");

        assert_eq!(path.length, 3);
        assert!(rendered.contains("ClothingItem.clothingSessionId = ClothingSession.id"));
        assert!(rendered.contains("-> ClothingSession.sellerProfileId = SellerProfile.id"));
        assert!(rendered.contains("-> SellerProfile.userId = User.id"));
    }
}
