//! Integration tests for the PostgreSQL DDL -> metadata parser.
//!
//! These tests cover the shapes found in real `pg_dump` output and in
//! hand-written migration files: inline and table-level foreign keys, composite
//! keys, `ALTER TABLE` additions and drops, quoted mixed-case identifiers,
//! several schemas, and the odd comments and dollar-quoted bodies that must not
//! confuse the statement splitter.

use schema_pathfinder::pathfinder_core::{load_schema_from_sql, ForeignKeyEdge};
use serde_json::Value;

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

fn edge_names(schema: &schema_pathfinder::pathfinder_core::SchemaMetadata) -> Vec<String> {
    schema
        .edges
        .iter()
        .map(|edge| {
            format!(
                "{}.{}.{} -> {}.{}.{}",
                edge.from.schema, edge.from.table, edge.from_column, edge.to.schema, edge.to.table,
                edge.to_column
            )
        })
        .collect()
}

fn table_names(schema: &schema_pathfinder::pathfinder_core::SchemaMetadata) -> Vec<String> {
    schema
        .tables
        .iter()
        .map(|table| format!("{}.{}", table.schema, table.table))
        .collect()
}

/// Path of the example DDL dump, relative to the repository root.
///
/// Cargo runs integration tests from the package directory, so the path is
/// anchored on `CARGO_MANIFEST_DIR` instead of the current directory.
fn repository_fixture(relative: &str) -> String {
    format!(
        "{}/../../fixtures/postgres/{}",
        env!("CARGO_MANIFEST_DIR"),
        relative
    )
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

// ---- Real-world DDL shapes -------------------------------------------------

#[test]
fn parses_the_example_dump_like_the_seed_fixture() {
    let sql = std::fs::read_to_string(repository_fixture("dressshot_schema.sql"))
        .expect("example dump is readable");
    let seed: Value = serde_json::from_str(
        &std::fs::read_to_string(repository_fixture("dressshot_seed_fk_edges.json"))
            .expect("seed fixture is readable"),
    )
    .expect("seed fixture is valid JSON");
    let expected = seed["edges"].as_array().expect("seed edges");

    let schema = load_schema_from_sql(&sql).expect("sql loads");

    assert_eq!(schema.edges.len(), expected.len());

    for expected_edge in expected {
        let constraint = expected_edge["constraintName"]
            .as_str()
            .expect("constraintName");
        let parsed = find_edge(&schema.edges, constraint);

        assert_eq!(
            parsed.from.schema,
            expected_edge["from"]["schema"].as_str().expect("from schema")
        );
        assert_eq!(
            parsed.from.table,
            expected_edge["from"]["table"].as_str().expect("from table")
        );
        assert_eq!(
            parsed.from_column,
            expected_edge["fromColumn"].as_str().expect("fromColumn")
        );
        assert_eq!(
            parsed.to.schema,
            expected_edge["to"]["schema"].as_str().expect("to schema")
        );
        assert_eq!(
            parsed.to.table,
            expected_edge["to"]["table"].as_str().expect("to table")
        );
        assert_eq!(
            parsed.to_column,
            expected_edge["toColumn"].as_str().expect("toColumn")
        );
    }
}

#[test]
fn marks_sql_edges_with_their_origin() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges[0].evidence, ["declared_fk", "sql_ddl"]);
}

#[test]
fn expands_a_composite_foreign_key_into_one_edge_per_column_pair() {
    let sql = r#"
        create table sales."OrderHead" ( "id" text, "number" int );
        create table sales."OrderLine" (
            "orderId" text,
            "lineNumber" int,
            constraint "OrderLine_order_fkey"
                foreign key ("orderId", "lineNumber") references sales."OrderHead"("id", "number")
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 2);
    assert_eq!(
        edge_names(&schema),
        [
            "sales.OrderLine.orderId -> sales.OrderHead.id",
            "sales.OrderLine.lineNumber -> sales.OrderHead.number"
        ]
    );
    assert_eq!(schema.edges[0].constraint_name, "OrderLine_order_fkey");
    assert_eq!(schema.edges[1].constraint_name, "OrderLine_order_fkey");
}

#[test]
fn assumes_the_referenced_column_is_id_when_the_ddl_omits_it() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges[0].to_column, "id");
}

#[test]
fn keeps_referential_actions_and_deferrability_out_of_the_way() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."Post" (
            id text primary key,
            "userId" text,
            constraint "Post_userId_fkey"
                foreign key ("userId") references public."User"(id)
                on delete cascade on update cascade deferrable initially deferred
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 1);
    assert_eq!(schema.edges[0].constraint_name, "Post_userId_fkey");
}

#[test]
fn parses_add_column_with_an_inline_reference() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."Post" ( id text primary key );
        alter table public."Post" add column "userId" text references public."User"(id);
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 1);
    assert_eq!(
        schema.edges[0].constraint_name,
        "Post_userId_User_id_fkey"
    );
}

#[test]
fn drops_the_edge_removed_by_alter_table() {
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

#[test]
fn resolves_unqualified_foreign_key_targets_to_public() {
    let sql = r#"
        create table public."Post" (
            id text primary key,
            "authorId" text references authors(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges[0].to.schema, "public");
    assert_eq!(schema.edges[0].to.table, "authors");
}

#[test]
fn keeps_the_last_two_parts_of_a_three_part_name() {
    let sql = r#"
        create table shop.public."Customer" ( id text primary key );
        create table shop.public."Order" (
            id text primary key,
            "customerId" text references shop.public."Customer"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(table_names(&schema), ["public.Customer", "public.Order"]);
    assert_eq!(schema.edges[0].to.schema, "public");
}

#[test]
fn follows_a_foreign_key_across_schemas() {
    let sql = r#"
        create table sales."Customer" ( id text primary key );
        create table billing."Invoice" (
            id text primary key,
            "customerId" text constraint "Invoice_customerId_fkey" references sales."Customer"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges[0].from.schema, "billing");
    assert_eq!(schema.edges[0].to.schema, "sales");
}

#[test]
fn keeps_a_self_referencing_foreign_key() {
    let sql = r#"
        create table public."Category" (
            id text primary key,
            "parentId" text constraint "Category_parentId_fkey" references public."Category"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 1);
    assert_eq!(schema.edges[0].from, schema.edges[0].to);
}

#[test]
fn collects_every_foreign_key_of_a_single_table() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."Team" ( id text primary key );
        create table public."Membership" (
            id text primary key,
            "userId" text constraint "Membership_userId_fkey" references public."User"(id),
            "teamId" text constraint "Membership_teamId_fkey" references public."Team"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(
        edge_names(&schema),
        [
            "public.Membership.userId -> public.User.id",
            "public.Membership.teamId -> public.Team.id"
        ]
    );
}

#[test]
fn lists_tables_that_only_appear_as_a_foreign_key_target() {
    let sql = r#"
        create table public."Post" (
            id text primary key,
            "authorId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(table_names(&schema), ["public.Post", "public.User"]);
}

#[test]
fn keeps_quoted_mixed_case_identifiers_as_written() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create table public."ClothingItem" (
            id text primary key,
            "ownerId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(table_names(&schema), ["public.ClothingItem", "public.User"]);
}

#[test]
fn accepts_uppercase_keywords_and_if_not_exists() {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS public."User" ( id text PRIMARY KEY );
        CREATE TABLE IF NOT EXISTS public."Post" (
            id text PRIMARY KEY,
            "userId" text CONSTRAINT "Post_userId_fkey" REFERENCES public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.tables.len(), 2);
    assert_eq!(schema.edges.len(), 1);
}

#[test]
fn ignores_primary_key_unique_and_check_constraints() {
    let sql = r#"
        create table public."User" (
            id text primary key,
            email text not null,
            constraint "User_email_key" unique (email),
            constraint "User_email_check" check (email <> '')
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.tables.len(), 1);
    assert!(schema.edges.is_empty());
}

#[test]
fn ignores_drop_table_create_schema_and_create_index() {
    let sql = r#"
        create schema if not exists sales;
        create table public."User" ( id text primary key );
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"(id)
        );
        drop table if exists public."Legacy";
        create index "Post_userId_idx" on public."Post" ("userId");
        comment on table public."Post" is 'blog posts';
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(table_names(&schema), ["public.Post", "public.User"]);
    assert_eq!(schema.edges.len(), 1);
}

// ---- Lenient statement splitting ------------------------------------------

#[test]
fn skips_unparsable_statements_that_do_not_declare_a_foreign_key() {
    let sql = r#"
        create table public."User" ( id text primary key );
        this is not sql at all;
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.tables.len(), 2);
    assert_eq!(schema.edges.len(), 1);
}

#[test]
fn fails_when_an_unparsable_statement_looks_like_a_foreign_key() {
    let sql = r#"
        create table public."User" ( id text primary key );
        this is broken but mentions foreign key (userId);
    "#;

    let error = load_schema_from_sql(sql).expect_err("should fail");

    assert_eq!(error.code, "SQL_PARSE_FAILED");
}

#[test]
fn does_not_split_a_statement_on_a_semicolon_inside_a_comment() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create bogus statement;
        create table public."Post" (
            id text primary key,
            -- ; a note with a semicolon
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.tables.len(), 2);
    assert_eq!(schema.edges.len(), 1);
}

#[test]
fn does_not_split_a_statement_on_a_semicolon_inside_a_dollar_quoted_body() {
    let sql = r#"
        create table public."User" ( id text primary key );
        create function public."touch"() returns trigger as $body$
        begin
            -- a note with ; and the word references
            return new;
        end;
        $body$ language plpgsql;
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.tables.len(), 2);
    assert_eq!(schema.edges.len(), 1);
}

#[test]
fn does_not_split_a_statement_on_a_semicolon_inside_a_string() {
    let sql = r#"
        create table public."User" ( id text primary key );
        comment on table public."User" is 'users; owners';
        create table public."Post" (
            id text primary key,
            "userId" text references public."User"(id)
        );
    "#;

    let schema = load_schema_from_sql(sql).expect("sql loads");

    assert_eq!(schema.edges.len(), 1);
}

#[test]
fn returns_an_empty_schema_for_an_empty_or_comment_only_script() {
    let schema = load_schema_from_sql("-- nothing to parse here\n").expect("sql loads");

    assert!(schema.tables.is_empty());
    assert!(schema.edges.is_empty());
}

#[test]
fn lists_tables_in_a_stable_sorted_order() {
    let schema = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
    let names = table_names(&schema);
    let mut sorted = names.clone();
    sorted.sort();

    assert_eq!(names, sorted);
    assert_eq!(
        names,
        [
            "public.ClothingItem",
            "public.ClothingSession",
            "public.SellerProfile",
            "public.User"
        ]
    );
}

#[test]
fn parses_the_same_ddl_to_the_same_edges_every_time() {
    let first = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");
    let second = load_schema_from_sql(DRESSSHOT_SQL).expect("sql loads");

    assert_eq!(edge_names(&first), edge_names(&second));
}
