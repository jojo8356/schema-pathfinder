//! Shared helpers for the integration tests.
//!
//! Each file in `tests/` is compiled as its own crate, so helpers that a given
//! test file does not use would trigger dead-code warnings. `tests/common/mod.rs`
//! is the conventional place for shared setup, and `#![allow(dead_code)]` keeps
//! the per-crate warnings quiet.
#![allow(dead_code)]

use schema_pathfinder::pathfinder_core::{ForeignKeyEdge, TableIdentifier};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// A `public.<name>` table identifier.
pub fn table(name: &str) -> TableIdentifier {
    TableIdentifier {
        schema: "public".to_string(),
        table: name.to_string(),
    }
}

/// A `<schema>.<name>` table identifier.
pub fn table_in(schema: &str, name: &str) -> TableIdentifier {
    TableIdentifier {
        schema: schema.to_string(),
        table: name.to_string(),
    }
}

/// A declared foreign-key edge in the `public` schema.
pub fn fk(
    constraint: &str,
    from: &str,
    from_column: &str,
    to: &str,
    to_column: &str,
) -> ForeignKeyEdge {
    ForeignKeyEdge {
        constraint_name: constraint.to_string(),
        from: table(from),
        from_column: from_column.to_string(),
        to: table(to),
        to_column: to_column.to_string(),
        evidence: vec!["declared_fk".to_string()],
    }
}

/// A schema with two independent routes from `A` to `D`:
///   * `A -> B -> D`        (2 links)
///   * `A -> C -> E -> D`   (3 links)
pub fn branching_edges() -> Vec<ForeignKeyEdge> {
    vec![
        fk("A_bId_fkey", "A", "bId", "B", "id"),
        fk("B_dId_fkey", "B", "dId", "D", "id"),
        fk("A_cId_fkey", "A", "cId", "C", "id"),
        fk("C_eId_fkey", "C", "eId", "E", "id"),
        fk("E_dId_fkey", "E", "dId", "D", "id"),
    ]
}

/// The dressshot chain `ClothingItem -> ClothingSession -> SellerProfile -> User`.
pub fn dressshot_edges() -> Vec<ForeignKeyEdge> {
    vec![
        fk(
            "ClothingItem_clothingSessionId_fkey",
            "ClothingItem",
            "clothingSessionId",
            "ClothingSession",
            "id",
        ),
        fk(
            "ClothingSession_sellerProfileId_fkey",
            "ClothingSession",
            "sellerProfileId",
            "SellerProfile",
            "id",
        ),
        fk(
            "SellerProfile_userId_fkey",
            "SellerProfile",
            "userId",
            "User",
            "id",
        ),
    ]
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Write `content` to a uniquely named temporary file and return its path.
/// Using the OS temp dir keeps the tests independent from the current working
/// directory (Cargo runs tests from the package root).
pub fn write_temp_file(prefix: &str, content: &str) -> PathBuf {
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let file_name = format!(
        "schema_pathfinder_test_{}_{}_{}",
        prefix,
        std::process::id(),
        unique
    );
    let path = env::temp_dir().join(file_name);
    fs::write(&path, content).expect("temp file write");
    path
}
