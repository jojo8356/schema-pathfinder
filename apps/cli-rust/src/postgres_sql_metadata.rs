//! PostgreSQL DDL → [`SchemaMetadata`] parser.
//!
//! [`load_schema_from_sql`] parses a PostgreSQL DDL dump with `sqlparser` and
//! keeps only what the pathfinder needs: the tables and the **declared**
//! foreign keys. Supported shapes:
//!
//! - `CREATE TABLE` with an inline column reference
//!   (`"userId" text REFERENCES public."User"(id)`);
//! - `CREATE TABLE` with a table-level constraint
//!   (`CONSTRAINT x FOREIGN KEY (a) REFERENCES t(b)`), including composite keys;
//! - `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ...`;
//! - `ALTER TABLE ... ADD COLUMN ... REFERENCES ...`;
//! - `ALTER TABLE ... DROP CONSTRAINT ...`, which removes the matching edge;
//! - qualified (`schema.table`) and unqualified names, quoted or not.
//!
//! Documented limits, all of them covered by tests:
//!
//! - a foreign key whose target is **not** schema-qualified resolves to
//!   `public`, matching PostgreSQL's default `search_path`;
//! - a composite foreign key becomes **one edge per column pair**;
//! - when the DDL omits the referenced column, `id` is assumed;
//! - when the DDL omits the constraint name, a PostgreSQL-style
//!   `<table>_<column>_<table>_<column>_fkey` name is generated;
//! - statements `sqlparser` cannot read are skipped, **unless** the failing
//!   statement looks like it declares a foreign key, in which case parsing
//!   fails with `SQL_PARSE_FAILED` rather than silently losing an edge.
//!
//! The parser never inspects rows and never connects to a database.

use crate::pathfinder_core::{
    pathfinder_error, ForeignKeyEdge, PathfinderError, SchemaMetadata, TableIdentifier,
};
use sqlparser::ast::{
    AlterTableOperation, ColumnOption, Ident, ObjectName, Statement, TableConstraint,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::collections::BTreeMap;

/// Referenced column assumed when the DDL writes `REFERENCES table` with no
/// column list. PostgreSQL itself requires the column, but hand-written dumps
/// sometimes omit it.
const DEFAULT_REFERRED_COLUMN: &str = "id";

/// Schema assumed for a foreign-key target that is not schema-qualified.
const DEFAULT_SCHEMA: &str = "public";

/// Parse PostgreSQL DDL text into a [`SchemaMetadata`].
///
/// # Errors
///
/// `SQL_PARSE_FAILED` when no statement of the input can be parsed, or when a
/// statement that looks like a foreign-key declaration cannot be parsed.
pub fn load_schema_from_sql(sql: &str) -> Result<SchemaMetadata, PathfinderError> {
    let dialect = PostgreSqlDialect {};
    let statements = parse_postgres_statements(sql, &dialect)?;
    let mut tables: BTreeMap<String, TableIdentifier> = BTreeMap::new();
    let mut edges = Vec::new();

    for statement in statements {
        collect_statement_metadata(&statement, &mut tables, &mut edges);
    }

    // A foreign key may point at a table that is not part of the dump (or whose
    // CREATE TABLE could not be parsed). Listing it keeps the graph complete.
    for edge in &edges {
        tables.insert(table_key(&edge.from), edge.from.clone());
        tables.insert(table_key(&edge.to), edge.to.clone());
    }

    Ok(SchemaMetadata {
        tables: tables.into_values().collect(),
        edges,
    })
}

fn parse_postgres_statements(
    sql: &str,
    dialect: &PostgreSqlDialect,
) -> Result<Vec<Statement>, PathfinderError> {
    match Parser::parse_sql(dialect, sql) {
        Ok(statements) => Ok(statements),
        Err(first_error) => parse_postgres_statements_lenient(sql, dialect, first_error),
    }
}

/// Parse the statements one by one so that a single exotic statement does not
/// throw away the whole dump.
fn parse_postgres_statements_lenient(
    sql: &str,
    dialect: &PostgreSqlDialect,
    first_error: sqlparser::parser::ParserError,
) -> Result<Vec<Statement>, PathfinderError> {
    let mut statements = Vec::new();

    for raw_statement in split_sql_statements(sql) {
        let trimmed = raw_statement.trim();

        if trimmed.is_empty() {
            continue;
        }

        match Parser::parse_sql(dialect, trimmed) {
            Ok(mut parsed) => statements.append(&mut parsed),
            Err(error) => {
                if statement_may_define_foreign_key(trimmed) {
                    return Err(pathfinder_error("SQL_PARSE_FAILED", error));
                }
            }
        }
    }

    if statements.is_empty() {
        return Err(pathfinder_error("SQL_PARSE_FAILED", first_error));
    }

    Ok(statements)
}

/// Split a SQL script on the `;` that actually terminate a statement.
///
/// Semicolons inside single-quoted strings, double-quoted identifiers,
/// dollar-quoted bodies (`$$ ... $$`, `$tag$ ... $tag$`), line comments and
/// block comments are part of the statement, not separators.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let characters: Vec<char> = sql.chars().collect();
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        if character == '-' && characters.get(index + 1) == Some(&'-') {
            while index < characters.len() && characters[index] != '\n' {
                current.push(characters[index]);
                index += 1;
            }

            continue;
        }

        if character == '/' && characters.get(index + 1) == Some(&'*') {
            current.push_str("/*");
            index += 2;

            while index < characters.len() {
                if characters[index] == '*' && characters.get(index + 1) == Some(&'/') {
                    current.push_str("*/");
                    index += 2;
                    break;
                }

                current.push(characters[index]);
                index += 1;
            }

            continue;
        }

        if character == '\'' {
            index = copy_quoted(&characters, index, '\'', &mut current);
            continue;
        }

        if character == '"' {
            index = copy_quoted(&characters, index, '"', &mut current);
            continue;
        }

        if character == '$' {
            if let Some(tag) = dollar_tag_at(&characters, index) {
                let closing: Vec<char> = tag.chars().collect();
                current.push_str(&tag);
                index += closing.len();

                while index < characters.len() {
                    if characters[index..].starts_with(&closing) {
                        current.push_str(&tag);
                        index += closing.len();
                        break;
                    }

                    current.push(characters[index]);
                    index += 1;
                }

                continue;
            }
        }

        if character == ';' {
            statements.push(std::mem::take(&mut current));
            index += 1;
            continue;
        }

        current.push(character);
        index += 1;
    }

    if !current.trim().is_empty() {
        statements.push(current);
    }

    statements
}

/// Copy a `'...'` or `"..."` run verbatim, honouring the doubled-quote escape,
/// and return the index just after it.
fn copy_quoted(characters: &[char], start: usize, quote: char, current: &mut String) -> usize {
    current.push(quote);
    let mut index = start + 1;

    while index < characters.len() {
        let character = characters[index];
        current.push(character);

        if character != quote {
            index += 1;
            continue;
        }

        if characters.get(index + 1) == Some(&quote) {
            current.push(quote);
            index += 2;
            continue;
        }

        return index + 1;
    }

    index
}

/// Return the dollar-quote opener starting at `index` (`$$`, `$body$`, ...), if
/// any. `$1` style placeholders are not dollar quotes.
fn dollar_tag_at(characters: &[char], index: usize) -> Option<String> {
    if characters.get(index) != Some(&'$') {
        return None;
    }

    let mut end = index + 1;

    while end < characters.len() {
        let character = characters[end];

        if character == '$' {
            return Some(characters[index..=end].iter().collect());
        }

        if character.is_ascii_digit() && end == index + 1 {
            return None;
        }

        if character.is_ascii_alphanumeric() || character == '_' {
            end += 1;
            continue;
        }

        return None;
    }

    None
}

/// `true` when a statement that `sqlparser` rejects still looks like it
/// declares a foreign key. Comments are ignored, so a commented-out FK does not
/// turn an unrelated parse error into a hard failure.
fn statement_may_define_foreign_key(statement: &str) -> bool {
    let code = strip_sql_comments(statement).to_lowercase();

    code.contains("foreign key") || code.contains("references")
}

/// Remove line and block comments from a SQL fragment, keeping strings intact.
fn strip_sql_comments(sql: &str) -> String {
    let characters: Vec<char> = sql.chars().collect();
    let mut stripped = String::new();
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        if character == '-' && characters.get(index + 1) == Some(&'-') {
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }

            continue;
        }

        if character == '/' && characters.get(index + 1) == Some(&'*') {
            index += 2;

            while index < characters.len() {
                if characters[index] == '*' && characters.get(index + 1) == Some(&'/') {
                    index += 2;
                    break;
                }

                index += 1;
            }

            continue;
        }

        if character == '\'' || character == '"' {
            index = copy_quoted(&characters, index, character, &mut stripped);
            continue;
        }

        stripped.push(character);
        index += 1;
    }

    stripped
}

fn collect_statement_metadata(
    statement: &Statement,
    tables: &mut BTreeMap<String, TableIdentifier>,
    edges: &mut Vec<ForeignKeyEdge>,
) {
    match statement {
        Statement::CreateTable(create_table) => {
            let table = table_identifier_from_object_name(&create_table.name);
            tables.insert(table_key(&table), table.clone());

            for column in &create_table.columns {
                collect_column_foreign_keys(&table, column, edges);
            }

            for constraint in &create_table.constraints {
                collect_table_constraint_foreign_key(&table, constraint, edges);
            }
        }
        Statement::AlterTable {
            name, operations, ..
        } => {
            let table = table_identifier_from_object_name(name);
            tables.insert(table_key(&table), table.clone());

            for operation in operations {
                collect_alter_table_operation(&table, operation, edges);
            }
        }
        _ => {}
    }
}

fn collect_alter_table_operation(
    table: &TableIdentifier,
    operation: &AlterTableOperation,
    edges: &mut Vec<ForeignKeyEdge>,
) {
    match operation {
        AlterTableOperation::AddConstraint(constraint) => {
            collect_table_constraint_foreign_key(table, constraint, edges);
        }
        AlterTableOperation::AddColumn { column_def, .. } => {
            collect_column_foreign_keys(table, column_def, edges);
        }
        AlterTableOperation::DropConstraint { name, .. } => {
            edges.retain(|edge| edge.constraint_name != name.value);
        }
        _ => {}
    }
}

fn collect_column_foreign_keys(
    table: &TableIdentifier,
    column: &sqlparser::ast::ColumnDef,
    edges: &mut Vec<ForeignKeyEdge>,
) {
    for option in &column.options {
        if let ColumnOption::ForeignKey {
            foreign_table,
            referred_columns,
            ..
        } = &option.option
        {
            let referred_column = first_referred_column(referred_columns);
            let target = table_identifier_from_object_name(foreign_table);
            let constraint_name = option_name_or_default(
                &option.name,
                table,
                &column.name.value,
                &target,
                &referred_column,
            );

            edges.push(ForeignKeyEdge {
                constraint_name,
                from: table.clone(),
                from_column: column.name.value.clone(),
                to: target,
                to_column: referred_column,
                evidence: vec!["declared_fk".to_string(), "sql_ddl".to_string()],
            });
        }
    }
}

fn collect_table_constraint_foreign_key(
    table: &TableIdentifier,
    constraint: &TableConstraint,
    edges: &mut Vec<ForeignKeyEdge>,
) {
    if let TableConstraint::ForeignKey {
        name,
        columns,
        foreign_table,
        referred_columns,
        ..
    } = constraint
    {
        let target = table_identifier_from_object_name(foreign_table);

        // A composite foreign key becomes one edge per column pair: the path
        // search works on single-column joins, and each pair is a real join
        // condition of the same constraint.
        for (index, column) in columns.iter().enumerate() {
            let referred_column = referred_column_at(referred_columns, index);
            let constraint_name =
                option_name_or_default(name, table, &column.value, &target, &referred_column);

            edges.push(ForeignKeyEdge {
                constraint_name,
                from: table.clone(),
                from_column: column.value.clone(),
                to: target.clone(),
                to_column: referred_column,
                evidence: vec!["declared_fk".to_string(), "sql_ddl".to_string()],
            });
        }
    }
}

fn first_referred_column(columns: &[Ident]) -> String {
    if let Some(column) = columns.first() {
        return column.value.clone();
    }

    DEFAULT_REFERRED_COLUMN.to_string()
}

fn referred_column_at(columns: &[Ident], index: usize) -> String {
    if let Some(column) = columns.get(index) {
        return column.value.clone();
    }

    first_referred_column(columns)
}

fn option_name_or_default(
    name: &Option<Ident>,
    from: &TableIdentifier,
    from_column: &str,
    to: &TableIdentifier,
    to_column: &str,
) -> String {
    if let Some(identifier) = name {
        return identifier.value.clone();
    }

    format!(
        "{}_{}_{}_{}_fkey",
        from.table, from_column, to.table, to_column
    )
}

fn table_identifier_from_object_name(name: &ObjectName) -> TableIdentifier {
    let parts = object_name_parts(name);

    // A three-part name (`database.schema.table`) keeps its last two parts.
    if parts.len() >= 2 {
        let schema_index = parts.len() - 2;
        let table_index = parts.len() - 1;

        return TableIdentifier {
            schema: parts[schema_index].clone(),
            table: parts[table_index].clone(),
        };
    }

    if let Some(table) = parts.first() {
        return TableIdentifier {
            schema: DEFAULT_SCHEMA.to_string(),
            table: table.clone(),
        };
    }

    TableIdentifier {
        schema: DEFAULT_SCHEMA.to_string(),
        table: "unknown".to_string(),
    }
}

fn object_name_parts(name: &ObjectName) -> Vec<String> {
    name.0.iter().map(|part| part.value.clone()).collect()
}

fn table_key(table: &TableIdentifier) -> String {
    format!("{}.{}", table.schema, table.table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_statements_on_terminating_semicolons() {
        let statements = split_sql_statements("select 1; select 2;");

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].trim(), "select 1");
        assert_eq!(statements[1].trim(), "select 2");
    }

    #[test]
    fn keeps_a_trailing_statement_without_semicolon() {
        let statements = split_sql_statements("select 1;\nselect 2");

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[1].trim(), "select 2");
    }

    #[test]
    fn ignores_semicolons_inside_strings_and_identifiers() {
        let statements = split_sql_statements(
            r#"select 'a;b' as x, "we;ird" as y; create table t ( id int );"#,
        );

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("'a;b'"));
        assert!(statements[0].contains("\"we;ird\""));
        assert_eq!(statements[1].trim(), "create table t ( id int )");
    }

    #[test]
    fn ignores_semicolons_inside_comments() {
        let statements = split_sql_statements(
            "-- a;b comment\nselect 1;\n/* another ; one */\nselect 2;",
        );

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("a;b comment"));
        assert!(statements[1].contains("another ; one"));
    }

    #[test]
    fn ignores_semicolons_inside_dollar_quoted_bodies() {
        let statements = split_sql_statements(
            "create function f() returns trigger as $$ begin; return new; end; $$ language plpgsql;\nselect 1;",
        );

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("begin; return new; end;"));
        assert_eq!(statements[1].trim(), "select 1");
    }

    #[test]
    fn ignores_semicolons_inside_tagged_dollar_quotes() {
        let statements = split_sql_statements("select $body$a;b$body$; select 2;");

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("$body$a;b$body$"));
    }

    #[test]
    fn recognizes_dollar_quote_openers_but_not_placeholders() {
        let characters: Vec<char> = "$$".chars().collect();
        assert_eq!(dollar_tag_at(&characters, 0), Some("$$".to_string()));

        let tagged: Vec<char> = "$body$".chars().collect();
        assert_eq!(dollar_tag_at(&tagged, 0), Some("$body$".to_string()));

        let placeholder: Vec<char> = "$1".chars().collect();
        assert_eq!(dollar_tag_at(&placeholder, 0), None);

        let arithmetic: Vec<char> = "$ 1".chars().collect();
        assert_eq!(dollar_tag_at(&arithmetic, 0), None);
    }

    #[test]
    fn strip_sql_comments_keeps_quoted_text() {
        assert_eq!(
            strip_sql_comments("-- drop me\nselect 'keep; -- me';"),
            "\nselect 'keep; -- me';"
        );
        assert_eq!(strip_sql_comments("a /* b */ c"), "a  c");
    }

    #[test]
    fn detects_foreign_key_declarations_outside_comments() {
        assert!(statement_may_define_foreign_key(
            "alter table a add constraint c foreign key (b) references d(e)"
        ));
        assert!(statement_may_define_foreign_key(
            "create table a ( b int references\nd(e) )"
        ));
        assert!(!statement_may_define_foreign_key(
            "-- alter table a add foreign key (b) references d(e)\nselect 1"
        ));
        assert!(!statement_may_define_foreign_key(
            "/* references */ select 1"
        ));
    }

    #[test]
    fn falls_back_to_the_first_referred_column() {
        assert_eq!(first_referred_column(&[]), "id");
        assert_eq!(referred_column_at(&[], 3), "id");
    }

    #[test]
    fn generates_postgres_style_constraint_names() {
        let from = TableIdentifier {
            schema: "public".to_string(),
            table: "Order".to_string(),
        };
        let to = TableIdentifier {
            schema: "public".to_string(),
            table: "Customer".to_string(),
        };

        assert_eq!(
            option_name_or_default(&None, &from, "customerId", &to, "id"),
            "Order_customerId_Customer_id_fkey"
        );
        assert_eq!(
            option_name_or_default(
                &Some(Ident::new("given_name")),
                &from,
                "customerId",
                &to,
                "id"
            ),
            "given_name"
        );
    }

    #[test]
    fn drop_constraint_removes_the_matching_edge() {
        let sql = r#"
            create table public."User" ( id text primary key );
            create table public."Post" (
                id text primary key,
                "userId" text constraint "Post_userId_fkey" references public."User"(id)
            );
            alter table public."Post" drop constraint "Post_userId_fkey";
        "#;

        let schema = load_schema_from_sql(sql).expect("sql loads");

        assert!(schema.edges.is_empty());
        assert_eq!(schema.tables.len(), 2);
    }
}
