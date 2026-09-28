//! Integration tests for the ASCII architecture-tree renderer.

mod common;

use common::table_in;
use schema_pathfinder::pathfinder_core::{
    render_postgres_architecture_tree, DatabaseArchitecture, ForeignKeyEdge,
    PostgresArchitectureTree, SchemaArchitecture, TableArchitecture,
};

fn sample_edge() -> ForeignKeyEdge {
    ForeignKeyEdge {
        constraint_name: "Order_customerId_fkey".to_string(),
        from: table_in("sales", "Order"),
        from_column: "customerId".to_string(),
        to: table_in("sales", "Customer"),
        to_column: "id".to_string(),
        evidence: vec!["declared_fk".to_string()],
    }
}

fn sample_tree() -> PostgresArchitectureTree {
    PostgresArchitectureTree {
        databases: vec![DatabaseArchitecture {
            name: "shop".to_string(),
            schemas: vec![SchemaArchitecture {
                name: "sales".to_string(),
                tables: vec![
                    TableArchitecture {
                        name: "Order".to_string(),
                        outgoing: vec![sample_edge()],
                    },
                    TableArchitecture {
                        name: "Customer".to_string(),
                        outgoing: vec![],
                    },
                ],
            }],
            error: None,
        }],
    }
}

#[test]
fn renders_databases_schemas_tables_and_foreign_keys() {
    let output = render_postgres_architecture_tree(&sample_tree());

    assert!(output.starts_with("postgres"));
    assert!(output.contains("database shop"));
    assert!(output.contains("schema sales"));
    assert!(output.contains("table Order"));
    assert!(output.contains("fk customerId -> sales.Customer.id (Order_customerId_fkey)"));
    assert!(output.contains("|--") || output.contains("`--"));
}

#[test]
fn empty_tree_reports_no_databases() {
    let tree = PostgresArchitectureTree { databases: vec![] };
    assert_eq!(render_postgres_architecture_tree(&tree), "NO_DATABASES");
}

#[test]
fn database_error_is_shown_in_the_tree() {
    let tree = PostgresArchitectureTree {
        databases: vec![DatabaseArchitecture {
            name: "broken".to_string(),
            schemas: vec![],
            error: Some("DB_CONNECT_FAILED: nope".to_string()),
        }],
    };

    let output = render_postgres_architecture_tree(&tree);
    assert!(output.contains("database broken"));
    assert!(output.contains("error DB_CONNECT_FAILED: nope"));
}

#[test]
fn database_without_schemas_reports_no_schemas() {
    let tree = PostgresArchitectureTree {
        databases: vec![DatabaseArchitecture {
            name: "empty".to_string(),
            schemas: vec![],
            error: None,
        }],
    };

    assert!(render_postgres_architecture_tree(&tree).contains("NO_SCHEMAS"));
}
