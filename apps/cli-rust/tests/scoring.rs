//! Integration tests for deterministic path scoring, exercised through the
//! public path API (scores are an observable property of returned paths).

mod common;

use common::{dressshot_edges, fk};
use schema_pathfinder::pathfinder_core::best_path;

#[test]
fn single_declared_fk_scores_sixty_minus_one_hop() {
    let edges = vec![fk("A_dId_fkey", "A", "dId", "D", "id")];
    let path = best_path(&edges, "A", "D").expect("path");

    // 60 for the declared FK, -8 for the single hop.
    assert_eq!(path.score, 52);
    assert_eq!(path.length, 1);
    assert_eq!(path.evidence, vec!["declared_fk".to_string()]);
}

#[test]
fn declared_chain_accumulates_score_with_hop_penalty() {
    let path = best_path(&dressshot_edges(), "ClothingItem", "User").expect("path");

    // 3 x 60 declared - 3 x 8 hops = 156.
    assert_eq!(path.score, 156);
    assert_eq!(path.score_contributions.first().expect("first").label, "declared_fk");
    assert_eq!(path.score_contributions.first().expect("first").value, 180);
    assert!(path
        .score_contributions
        .iter()
        .any(|item| item.label == "extra_hop" && item.value == -24));
}

#[test]
fn technical_and_auth_session_tables_are_penalized() {
    let edges = vec![
        fk("A_sessionId_fkey", "A", "sessionId", "Session", "id"),
        fk(
            "Session_migrationId_fkey",
            "Session",
            "migrationId",
            "_prisma_migrations",
            "id",
        ),
    ];

    let path = best_path(&edges, "A", "_prisma_migrations").expect("path");

    // 2 x 60 declared - 2 x 8 hops - 15 auth/session - 25 technical = 64.
    assert_eq!(path.score, 64);
    assert!(path
        .score_contributions
        .iter()
        .any(|item| item.label == "auth_session_table" && item.value == -15));
    assert!(path
        .score_contributions
        .iter()
        .any(|item| item.label == "technical_table" && item.value == -25));
}

#[test]
fn a_penalized_table_is_only_charged_once() {
    // Session appears twice as an endpoint but must be penalized a single time.
    let edges = vec![
        fk("A_sessionId_fkey", "A", "sessionId", "Session", "id"),
        fk("Session_bId_fkey", "Session", "bId", "B", "id"),
    ];

    let path = best_path(&edges, "A", "B").expect("path");

    let auth_penalties = path
        .score_contributions
        .iter()
        .filter(|item| item.label == "auth_session_table")
        .count();

    assert_eq!(auth_penalties, 1);
    // 2 x 60 - 2 x 8 - 15 = 89.
    assert_eq!(path.score, 89);
}
