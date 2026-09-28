//! # Schema Pathfinder
//!
//! Schema Pathfinder is a standalone tool that answers one concrete question:
//!
//! > *How do I join table A to table B?*
//!
//! It works purely on **schema metadata** — tables, schemas and declared
//! foreign keys — and never reads business rows. From that metadata it can:
//!
//! - list tables, schemas and databases;
//! - render a full architecture tree of a PostgreSQL server;
//! - find the foreign-key path(s) between two tables and render them as text,
//!   join equations, read-only SQL, Mermaid or JSON.
//!
//! ## Where to start
//!
//! Everything reusable lives in [`pathfinder_core`]. Typical entry points:
//!
//! - [`pathfinder_core::load_schema_from_fixture`],
//!   [`pathfinder_core::load_schema_from_sql`] and
//!   [`pathfinder_core::discover_postgres_schema`] to obtain a
//!   [`pathfinder_core::SchemaMetadata`];
//! - [`pathfinder_core::ranked_paths_by_complexity`] to list every join path
//!   between two tables, simplest first;
//! - [`pathfinder_core::render_paths`] to turn those paths into a chosen output
//!   format.
//!
//! ```
//! use schema_pathfinder::pathfinder_core::{
//!     load_schema_from_sql, ranked_paths_by_complexity, render_paths, parse_format,
//!     DEFAULT_MAX_LINKS,
//! };
//!
//! let sql = r#"
//!     create table public."User" ( id text primary key );
//!     create table public."Post" (
//!         id text primary key,
//!         "userId" text constraint "Post_userId_fkey" references public."User"(id)
//!     );
//! "#;
//!
//! let schema = load_schema_from_sql(sql).expect("valid DDL");
//! let paths = ranked_paths_by_complexity(&schema.edges, "Post", "User", DEFAULT_MAX_LINKS)
//!     .expect("a path exists");
//! let rendered = render_paths(&paths, parse_format("equation").unwrap()).unwrap();
//! assert!(rendered.contains("Post.userId = User.id"));
//! ```

/// Core library: schema metadata models, PostgreSQL/SQL/fixture loaders, path
/// search, scoring and output renderers. This is the public API used by the
/// CLI, the desktop app and the web API.
pub mod pathfinder_core;
mod postgres_sql_metadata;
