use crate::pathfinder_core::{
    pathfinder_error, ForeignKeyEdge, PathfinderError, SchemaMetadata, TableIdentifier,
};
use sqlparser::ast::{
    AlterTableOperation, ColumnOption, Ident, ObjectName, Statement, TableConstraint,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use std::collections::BTreeMap;

pub fn load_schema_from_sql(sql: &str) -> Result<SchemaMetadata, PathfinderError> {
    let dialect = PostgreSqlDialect {};
    let statements = parse_postgres_statements(sql, &dialect)?;
    let mut tables: BTreeMap<String, TableIdentifier> = BTreeMap::new();
    let mut edges = Vec::new();

    for statement in statements {
        collect_statement_metadata(&statement, &mut tables, &mut edges);
    }

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

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(character) = chars.next() {
        if in_line_comment {
            current.push(character);

            if character == '\n' {
                in_line_comment = false;
            }

            continue;
        }

        if in_block_comment {
            current.push(character);

            if character == '*' {
                if let Some('/') = chars.peek() {
                    current.push('/');
                    chars.next();
                    in_block_comment = false;
                }
            }

            continue;
        }

        if in_single_quote {
            current.push(character);

            if character == '\'' {
                if let Some('\'') = chars.peek() {
                    current.push('\'');
                    chars.next();
                } else {
                    in_single_quote = false;
                }
            }

            continue;
        }

        if in_double_quote {
            current.push(character);

            if character == '"' {
                in_double_quote = false;
            }

            continue;
        }

        if character == '-' {
            if let Some('-') = chars.peek() {
                current.push(character);
                current.push('-');
                chars.next();
                in_line_comment = true;
                continue;
            }
        }

        if character == '/' {
            if let Some('*') = chars.peek() {
                current.push(character);
                current.push('*');
                chars.next();
                in_block_comment = true;
                continue;
            }
        }

        if character == '\'' {
            current.push(character);
            in_single_quote = true;
            continue;
        }

        if character == '"' {
            current.push(character);
            in_double_quote = true;
            continue;
        }

        if character == ';' {
            statements.push(current.clone());
            current.clear();
            continue;
        }

        current.push(character);
    }

    if !current.trim().is_empty() {
        statements.push(current);
    }

    statements
}

fn statement_may_define_foreign_key(statement: &str) -> bool {
    let lower = statement.to_lowercase();
    lower.contains("foreign key") || lower.contains(" references ")
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

    "id".to_string()
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
            schema: "public".to_string(),
            table: table.clone(),
        };
    }

    TableIdentifier {
        schema: "public".to_string(),
        table: "unknown".to_string(),
    }
}

fn object_name_parts(name: &ObjectName) -> Vec<String> {
    name.0.iter().map(|part| part.value.clone()).collect()
}

fn table_key(table: &TableIdentifier) -> String {
    format!("{}.{}", table.schema, table.table)
}
