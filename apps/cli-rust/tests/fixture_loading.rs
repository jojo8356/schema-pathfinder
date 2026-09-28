//! Integration tests for loading metadata from fixture JSON and SQL files.

mod common;

use common::write_temp_file;
use schema_pathfinder::pathfinder_core::{
    load_edges_from_fixture, load_edges_from_sql_file, load_schema_from_fixture,
    load_schema_from_sql_file,
};

const FIXTURE_EDGES_ONLY: &str = r#"{
  "edges": [
    {"constraintName":"A_bId_fkey","from":{"schema":"public","table":"A"},"fromColumn":"bId","to":{"schema":"public","table":"B"},"toColumn":"id","evidence":["declared_fk"]},
    {"constraintName":"B_cId_fkey","from":{"schema":"public","table":"B"},"fromColumn":"cId","to":{"schema":"public","table":"C"},"toColumn":"id","evidence":["declared_fk"]}
  ]
}"#;

const FIXTURE_WITH_TABLES: &str = r#"{
  "tables": [
    {"schema":"public","table":"Only"}
  ],
  "edges": []
}"#;

#[test]
fn derives_tables_from_edges_when_absent() {
    let path = write_temp_file("edges_only.json", FIXTURE_EDGES_ONLY);
    let schema = load_schema_from_fixture(path.to_str().unwrap()).expect("fixture loads");

    assert_eq!(schema.edges.len(), 2);
    assert_eq!(schema.tables.len(), 3);
    assert!(schema.tables.iter().any(|table| table.table == "A"));
    assert!(schema.tables.iter().any(|table| table.table == "C"));
}

#[test]
fn uses_explicit_tables_when_provided() {
    let path = write_temp_file("with_tables.json", FIXTURE_WITH_TABLES);
    let schema = load_schema_from_fixture(path.to_str().unwrap()).expect("fixture loads");

    assert_eq!(schema.tables.len(), 1);
    assert_eq!(schema.tables[0].table, "Only");
    assert!(schema.edges.is_empty());
}

#[test]
fn load_edges_from_fixture_returns_only_edges() {
    let path = write_temp_file("edges.json", FIXTURE_EDGES_ONLY);
    let edges = load_edges_from_fixture(path.to_str().unwrap()).expect("fixture loads");

    assert_eq!(edges.len(), 2);
}

#[test]
fn reports_invalid_fixture_json() {
    let path = write_temp_file("broken.json", "{ this is not json ]");
    let error = load_schema_from_fixture(path.to_str().unwrap()).expect_err("should fail");

    assert_eq!(error.code, "FIXTURE_JSON_INVALID");
}

#[test]
fn reports_missing_fixture_file() {
    let error = load_schema_from_fixture("/nonexistent/path/fixture.json").expect_err("should fail");

    assert_eq!(error.code, "FIXTURE_READ_FAILED");
}

#[test]
fn loads_schema_and_edges_from_sql_file() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."SellerProfile" (
            id text primary key,
            "userId" text constraint "SellerProfile_userId_fkey" references public."User"(id)
        );
    "#;
    let path = write_temp_file("schema.sql", sql);

    let schema = load_schema_from_sql_file(path.to_str().unwrap()).expect("sql loads");
    assert_eq!(schema.edges.len(), 1);
    assert_eq!(schema.tables.len(), 2);

    let edges = load_edges_from_sql_file(path.to_str().unwrap()).expect("sql loads");
    assert_eq!(edges.len(), 1);
}

#[test]
fn reports_missing_sql_file() {
    let error = load_schema_from_sql_file("/nonexistent/path/schema.sql").expect_err("should fail");

    assert_eq!(error.code, "SQL_READ_FAILED");
}
