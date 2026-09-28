//! Integration tests for rewriting the database name inside a connection URL.

use schema_pathfinder::pathfinder_core::database_url_for_database;

#[test]
fn replaces_the_database_name_in_the_path() {
    assert_eq!(
        database_url_for_database("postgres://user:pass@localhost:5432/postgres", "shop"),
        "postgres://user:pass@localhost:5432/shop"
    );
}

#[test]
fn preserves_the_query_string() {
    assert_eq!(
        database_url_for_database(
            "postgres://user:pass@localhost:5432/postgres?sslmode=require",
            "shop"
        ),
        "postgres://user:pass@localhost:5432/shop?sslmode=require"
    );
}

#[test]
fn appends_a_database_name_when_the_url_has_no_path() {
    assert_eq!(
        database_url_for_database("postgres://user:pass@localhost:5432", "shop"),
        "postgres://user:pass@localhost:5432/shop"
    );
}

#[test]
fn returns_the_url_unchanged_when_there_is_no_scheme() {
    assert_eq!(
        database_url_for_database("localhost-only", "shop"),
        "localhost-only"
    );
}
