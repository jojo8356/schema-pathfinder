//! Integration tests for the output renderers (single path and multi-path).

mod common;

use common::{branching_edges, dressshot_edges, table};
use schema_pathfinder::pathfinder_core::{
    best_path, parse_format, ranked_paths_by_complexity, render_names, render_path, render_paths,
    render_tables, DEFAULT_MAX_LINKS,
};

fn dressshot_path() -> schema_pathfinder::pathfinder_core::ScoredPath {
    best_path(&dressshot_edges(), "ClothingItem", "User").expect("path")
}

#[test]
fn render_path_text_lists_header_order_and_edges() {
    let output = render_path(&dressshot_path(), parse_format("text").unwrap()).unwrap();

    assert!(output.contains("Path 1 score 156 declared_fk length 3"));
    assert!(output.contains("ClothingItem -> ClothingSession -> SellerProfile -> User"));
    assert!(output.contains("1. ClothingItem.clothingSessionId -> ClothingSession.id"));
}

#[test]
fn render_path_equation_chains_with_arrows() {
    let output = render_path(&dressshot_path(), parse_format("equation").unwrap()).unwrap();

    assert_eq!(
        output,
        "ClothingItem.clothingSessionId = ClothingSession.id\n-> ClothingSession.sellerProfileId = SellerProfile.id\n-> SellerProfile.userId = User.id"
    );
}

#[test]
fn render_path_sql_has_quoted_identifiers_and_limit() {
    let output = render_path(&dressshot_path(), parse_format("sql").unwrap()).unwrap();

    assert!(output.starts_with("select *"));
    assert!(output.contains("from \"ClothingItem\" t0"));
    assert!(output.contains("join \"User\" t3"));
    assert!(output.trim_end().ends_with("limit 50;"));
}

#[test]
fn render_path_mermaid_emits_flowchart_edges() {
    let output = render_path(&dressshot_path(), parse_format("mermaid").unwrap()).unwrap();

    assert!(output.contains("flowchart LR"));
    assert!(output.contains("SellerProfile --> User"));
}

#[test]
fn render_path_json_contains_score() {
    let output = render_path(&dressshot_path(), parse_format("json").unwrap()).unwrap();

    assert!(output.contains("\"score\": 156"));
}

#[test]
fn render_paths_numbers_every_path_in_text() {
    let paths =
        ranked_paths_by_complexity(&branching_edges(), "A", "D", DEFAULT_MAX_LINKS).unwrap();
    let output = render_paths(&paths, parse_format("text").unwrap()).unwrap();

    assert!(output.contains("Path 1 score"));
    assert!(output.contains("Path 2 score"));
}

#[test]
fn render_paths_equation_adds_a_header_per_path() {
    let paths =
        ranked_paths_by_complexity(&branching_edges(), "A", "D", DEFAULT_MAX_LINKS).unwrap();
    let output = render_paths(&paths, parse_format("equation").unwrap()).unwrap();

    assert!(output.contains("Path 1 score 104 length 2: A -> B -> D"));
    assert!(output.contains("Path 2 score 156 length 3: A -> C -> E -> D"));
    assert!(output.contains("A.bId = B.id"));
}

#[test]
fn render_paths_json_is_an_array() {
    let paths =
        ranked_paths_by_complexity(&branching_edges(), "A", "D", DEFAULT_MAX_LINKS).unwrap();
    let output = render_paths(&paths, parse_format("json").unwrap()).unwrap();

    assert!(output.trim_start().starts_with('['));
    assert_eq!(output.matches("\"length\"").count(), 2);
}

#[test]
fn render_paths_of_nothing_is_empty() {
    let output = render_paths(&[], parse_format("text").unwrap()).unwrap();
    assert_eq!(output, "");
}

#[test]
fn render_tables_lists_one_per_line_and_handles_empty() {
    let rendered = render_tables(&[table("ClothingItem"), table("User")]);
    assert_eq!(rendered, "public.ClothingItem\npublic.User");

    assert_eq!(render_tables(&[]), "NO_TABLES");
}

#[test]
fn render_names_joins_and_falls_back_to_empty_label() {
    assert_eq!(
        render_names(&["one".to_string(), "two".to_string()], "NONE"),
        "one\ntwo"
    );
    assert_eq!(render_names(&[], "NO_DATABASES"), "NO_DATABASES");
}
