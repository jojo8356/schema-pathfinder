//! Integration tests for path search: best path, ranked-by-complexity, limits.

mod common;

use common::{branching_edges, dressshot_edges, fk};
use schema_pathfinder::pathfinder_core::{
    best_path, clamp_max_links, ranked_paths_by_complexity, DEFAULT_MAX_LINKS, MAX_LINKS_CEILING,
};

#[test]
fn best_path_follows_declared_chain() {
    let edges = dressshot_edges();
    let path = best_path(&edges, "ClothingItem", "User").expect("path exists");

    assert_eq!(path.length, 3);
    assert_eq!(path.source.table, "ClothingItem");
    assert_eq!(path.edges.last().expect("last edge").to.table, "User");
    assert_eq!(
        path.edges
            .iter()
            .map(|edge| edge.constraint_name.clone())
            .collect::<Vec<String>>(),
        vec![
            "ClothingItem_clothingSessionId_fkey".to_string(),
            "ClothingSession_sellerProfileId_fkey".to_string(),
            "SellerProfile_userId_fkey".to_string(),
        ]
    );
}

#[test]
fn best_path_reports_missing_source_table() {
    let edges = dressshot_edges();
    let error = best_path(&edges, "DoesNotExist", "User").expect_err("should fail");

    assert_eq!(error.code, "TABLE_NOT_FOUND");
}

#[test]
fn best_path_reports_no_declared_fk_path() {
    let edges = dressshot_edges();
    // The chain only points ClothingItem -> ... -> User, never the reverse.
    let error = best_path(&edges, "User", "ClothingItem").expect_err("should fail");

    assert_eq!(error.code, "NO_DECLARED_FK_PATH");
}

#[test]
fn ranked_paths_are_ordered_from_simplest_to_most_complex() {
    let paths = ranked_paths_by_complexity(&branching_edges(), "A", "D", DEFAULT_MAX_LINKS)
        .expect("paths exist");

    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].length, 2);
    assert_eq!(paths[1].length, 3);
    assert_eq!(
        paths[0]
            .edges
            .iter()
            .map(|edge| edge.to.table.clone())
            .collect::<Vec<String>>(),
        vec!["B".to_string(), "D".to_string()]
    );
}

#[test]
fn ranked_paths_respect_the_max_links_limit() {
    let paths =
        ranked_paths_by_complexity(&branching_edges(), "A", "D", 2).expect("short path exists");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].length, 2);
}

#[test]
fn ranked_paths_with_one_link_find_nothing_when_two_are_required() {
    let error = ranked_paths_by_complexity(&branching_edges(), "A", "D", 1)
        .expect_err("no 1-link path exists");

    assert_eq!(error.code, "NO_DECLARED_FK_PATH");
}

#[test]
fn ranked_paths_order_equal_length_by_score() {
    // Two routes of the SAME length (2 links) from A to D:
    //   * A -> B -> D            (clean tables, score 104)
    //   * A -> Session -> D      (auth/session penalty, score 89)
    // At equal length the higher-scoring, cleaner route must come first.
    let edges = vec![
        fk("A_b_fkey", "A", "bId", "B", "id"),
        fk("B_d_fkey", "B", "dId", "D", "id"),
        fk("A_session_fkey", "A", "sessionId", "Session", "id"),
        fk("Session_d_fkey", "Session", "dId", "D", "id"),
    ];

    let paths = ranked_paths_by_complexity(&edges, "A", "D", DEFAULT_MAX_LINKS).expect("paths");

    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].length, 2);
    assert_eq!(paths[1].length, 2);
    assert_eq!(paths[0].score, 104);
    assert_eq!(paths[0].edges[0].to.table, "B");
    assert_eq!(paths[1].score, 89);
    assert_eq!(paths[1].edges[0].to.table, "Session");
}

#[test]
fn ranked_paths_report_missing_target_table() {
    let error = ranked_paths_by_complexity(&branching_edges(), "A", "Nope", DEFAULT_MAX_LINKS)
        .expect_err("missing target");

    assert_eq!(error.code, "TABLE_NOT_FOUND");
}

#[test]
fn clamp_max_links_bounds_the_requested_value() {
    assert_eq!(clamp_max_links(0), 1);
    assert_eq!(clamp_max_links(1), 1);
    assert_eq!(clamp_max_links(3), 3);
    assert_eq!(clamp_max_links(MAX_LINKS_CEILING), MAX_LINKS_CEILING);
    assert_eq!(clamp_max_links(MAX_LINKS_CEILING + 5), MAX_LINKS_CEILING);
}

#[test]
fn default_max_links_is_within_the_ceiling() {
    assert!(DEFAULT_MAX_LINKS >= 1);
    assert!(DEFAULT_MAX_LINKS <= MAX_LINKS_CEILING);
}
