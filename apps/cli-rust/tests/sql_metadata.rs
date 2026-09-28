//! Integration tests for the PostgreSQL DDL -> metadata parser.

use schema_pathfinder::pathfinder_core::{load_schema_from_sql, ForeignKeyEdge};

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

fn find_edge<'a>(edges: &'a [ForeignKeyEdge], constraint: &str) -> &'a ForeignKeyEdge {
    edges
        .iter()
        .find(|edge| edge.constraint_name == constraint)
        .unwrap_or_else(|| panic!("edge {} should exist", constraint))
}

#[test]
fn extracts_all_tables_and_foreign_keys() {
    let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");

    assert_eq!(schema.tables.len(), 4);
    assert_eq!(schema.edges.len(), 3);
}

#[test]
fn parses_inline_column_foreign_key() {
    let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
    let edge = find_edge(&schema.edges, "SellerProfile_userId_fkey");

    assert_eq!(edge.from.table, "SellerProfile");
    assert_eq!(edge.from_column, "userId");
    assert_eq!(edge.to.table, "User");
    assert_eq!(edge.to_column, "id");
    assert!(edge.evidence.contains(&"declared_fk".to_string()));
    assert!(edge.evidence.contains(&"sql_ddl".to_string()));
}

#[test]
fn parses_table_level_constraint_foreign_key() {
    let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
    let edge = find_edge(&schema.edges, "ClothingSession_sellerProfileId_fkey");

    assert_eq!(edge.from.table, "ClothingSession");
    assert_eq!(edge.from_column, "sellerProfileId");
    assert_eq!(edge.to.table, "SellerProfile");
}

#[test]
fn parses_alter_table_add_constraint_foreign_key() {
    let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
    let edge = find_edge(&schema.edges, "ClothingItem_clothingSessionId_fkey");

    assert_eq!(edge.from.table, "ClothingItem");
    assert_eq!(edge.to.table, "ClothingSession");
    assert_eq!(edge.to_column, "id");
}

#[test]
fn keeps_non_public_schema_names() {
    let sql = r#"
        create table sales."Customer" ( id text primary key );
        create table sales."Order" (
            id text primary key,
            "customerId" text constraint "Order_customerId_fkey" references sales."Customer"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");
    let order = schema
        .tables
        .iter()
        .find(|table| table.table == "Order")
        .expect("Order table");

    assert_eq!(order.schema, "sales");
    assert_eq!(schema.edges[0].from.schema, "sales");
    assert_eq!(schema.edges[0].to.schema, "sales");
}

#[test]
fn generates_a_default_constraint_name_when_missing() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."SellerProfile" (
            id text primary key,
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert!(schema
        .edges
        .iter()
        .any(|edge| edge.constraint_name == "SellerProfile_userId_User_id_fkey"));
}

#[test]
fn ignores_unrelated_statements() {
    let sql = r#"
        -- a leading line comment
        create table public."User" ( id text primary key );
        /* a block comment describing the next table */
        create table public."SellerProfile" (
            id text primary key,
            "userId" text constraint "SellerProfile_userId_fkey" references public."User"(id)
        );
        create index "SellerProfile_userId_idx" on public."SellerProfile" ("userId");
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 1);
    assert_eq!(schema.edges[0].constraint_name, "SellerProfile_userId_fkey");
}

#[test]
fn rejects_input_that_is_not_sql() {
    let error = load_schema_from_sql("this is definitely not sql").expect_err("should fail");
    assert_eq!(error.code, "SQL_PARSE_FAILED");
}
