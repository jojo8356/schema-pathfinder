# Complete tutorial: writing all of a project's tests

This guide explains, step by step, how to cover a project with **every useful
type of test**. It is written for this repository (`schema-pathfinder`, a Rust
core + Node/React surfaces), but the principles apply to any project.

The goal: move from a "works on my machine" project to one whose behavior is
**verified automatically**, on every commit, from the smallest detail (a private
function) up to the full user journey.

---

## 1. The philosophy: the testing pyramid

We distinguish three broad families, from smallest to widest:

1. **Unit tests** — verify a single, **isolated** function/module. Fast,
   numerous, and able to reach private code.
2. **Integration tests** — verify that several parts work together through the
   **public API**. Fewer, slower.
3. **End-to-end (E2E) tests** — verify the complete product (the CLI actually
   run, a real HTTP request, a UI in a browser). Few, slow.

Rule of thumb: many unit tests, a few integration tests, very few E2E tests.
Test the **observable behavior**, not the implementation details
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 2. The three kinds of tests in Rust

Rust natively supports **three** kinds of tests: unit, integration, and
documentation (doctests) [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

| Type | Location | Access | Compiled when |
| --- | --- | --- | --- |
| Unit | inside `src/*.rs`, `#[cfg(test)]` module | **private** code included | `cargo test` |
| Integration | `tests/` folder at the crate root | **public** API only | `cargo test` |
| Doctest | inside `///` comments | public API | `cargo test` |

---

## 3. Unit tests

They live **in the same file** as the code under test, inside a `tests` module
annotated with `#[cfg(test)]`. The `#[cfg(test)]` attribute means this code is
compiled and linked **only** during `cargo test`: it does not weigh on the
release binary and its test dependencies do not leak into production
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*; // imports the parent module, including private items

    #[test]
    fn adds_two_numbers() {
        assert_eq!(add(2, 2), 4);
    }
}
```

### Testing private functions

This is **the** superpower of unit tests: because they live inside the module,
`use super::*;` grants access to private functions. In this repository,
`apps/cli-rust/src/pathfinder_core.rs` tests private helpers this way
(`score_path`, `resolve_table`, `build_adjacency`, `quote_identifier`,
`clamp_max_links`, …):

```rust
#[test]
fn quote_identifier_doubles_embedded_quotes() {
    assert_eq!(quote_identifier("a\"b"), "\"a\"\"b\"");
}
```

Keep these tests **fast and I/O-free**. "If it touches the file system, it is
not a unit test"
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 4. Integration tests

They live in a **`tests/`** folder placed at the crate root (next to `src/` and
`Cargo.toml`). **Each `tests/*.rs` file is compiled as a separate crate** that
depends on your library the way an external user would: they therefore only see
the **public API**
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

```rust
// tests/path_search.rs
use schema_pathfinder::pathfinder_core::best_path;

#[test]
fn best_path_follows_declared_chain() {
    let edges = /* ... */;
    let path = best_path(&edges, "ClothingItem", "User").expect("path exists");
    assert_eq!(path.length, 3);
}
```

If you try to call a private function from `tests/`, the compiler refuses
(error `E0603`). This is **intentional**: it forces you to test the interface
your users actually consume
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

### Binary vs library tip

A **binary** crate (`main.rs` alone) exposes nothing: you cannot import it from
`tests/`. The good practice is to put the logic in `src/lib.rs` and keep a
minimal `main.rs` that calls it
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html). This is
exactly what this repository does: all the logic is in
`schema_pathfinder::pathfinder_core`, and `main.rs` / the API binary merely
orchestrate it.

### Sharing code between test files: `tests/common/mod.rs`

Because each file is a distinct crate, you cannot simply import a function from
another test file. The convention is to create **`tests/common/mod.rs`** (not
`tests/common.rs`, which would be seen as a test) and to `mod common;` in each
file [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

```rust
// tests/common/mod.rs
#![allow(dead_code)] // some helpers are not used in every file

pub fn table(name: &str) -> TableIdentifier { /* ... */ }
pub fn write_temp_file(prefix: &str, content: &str) -> PathBuf { /* ... */ }
```

```rust
// tests/fixture_loading.rs
mod common;
use common::write_temp_file;
```

> ⚠️ Classic pitfall: `src/tests/` is **not** an integration-test folder. Cargo
> only looks at `tests/` at the crate root. A `src/tests/` is treated as a plain
> module and your tests do not run
> [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

### Naming files by feature

Name `tests/path_search.rs`, `tests/renderers.rs`, `tests/sql_metadata.rs` …
after the **behavior** under test, not after an internal module
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 5. Doctests (tests in the documentation)

Any code block in a `///` comment is compiled **and executed** by `cargo test`.
This is perfect for guaranteeing that the examples in the docs stay correct.

```rust
/// Normalizes a "max links" value into the supported range.
///
/// ```
/// use schema_pathfinder::pathfinder_core::clamp_max_links;
/// assert_eq!(clamp_max_links(0), 1);
/// assert_eq!(clamp_max_links(99), 8);
/// ```
pub fn clamp_max_links(max_links: usize) -> usize { /* ... */ }
```

Tips:
- ` ```no_run ` compiles but does not run (useful for network/DB code).
- ` ```ignore ` is neither compiled nor run.
- ` ```should_panic ` checks that the example panics.
- A line prefixed with `# ` is hidden in the rendered docs but present at
  compile time.

---

## 6. Assertions and error tests

### Assertion macros

```rust
assert!(condition, "optional message {}", value);
assert_eq!(left, right);
assert_ne!(left, right);
```

### Testing that a function panics

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn panics_out_of_bounds() {
    let v = vec![1, 2, 3];
    let _ = v[10];
}
```

### Testing error paths (recommended for `Result`)

Rather than "unwrap" everywhere, explicitly check the error code:

```rust
#[test]
fn rejects_an_unknown_format() {
    let error = parse_format("yaml").expect_err("should fail");
    assert_eq!(error.code, "UNSUPPORTED_FORMAT");
}
```

### Tests that return `Result`

A test function can return `Result<(), E>`: you can then use `?`.

```rust
#[test]
fn loads_a_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let schema = load_schema_from_fixture("fixtures/x.json")?;
    assert!(!schema.edges.is_empty());
    Ok(())
}
```

### Temporarily ignoring a test

```rust
#[test]
#[ignore = "requires a local PostgreSQL database"]
fn queries_postgres() { /* ... */ }
```

You then run it with `cargo test -- --ignored`.

---

## 7. Fixtures, setup, and test data

- **Constants / builders**: prefer small *builders* (`fn table(...)`, `fn fk(...)`)
  instead of duplicating literals. See `apps/cli-rust/tests/common/mod.rs`.
- **Temporary files**: write into `std::env::temp_dir()` with a unique name
  (PID + atomic counter) to stay **independent of the current directory** (Cargo
  runs tests from the crate root) and **thread-safe** (tests run in parallel by
  default).
- **Setup/teardown**: Rust has no native `beforeEach`; you call a shared
  initialization function from `tests/common/mod.rs`
  [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 8. Parameterized tests with `rstest`

To run the same test on several sets of values without copy-pasting, the
**`rstest`** crate is the reference.

```toml
# Cargo.toml
[dev-dependencies]
rstest = "0.23"
```

```rust
use rstest::rstest;

#[rstest]
#[case(0, 1)]
#[case(5, 5)]
#[case(99, 8)]
fn clamp_bounds_the_values(#[case] input: usize, #[case] expected: usize) {
    assert_eq!(clamp_max_links(input), expected);
}
```

`rstest` also handles **fixtures** (`#[fixture]` functions injected as arguments).

---

## 9. Property-based tests

Instead of hand-picking examples, you describe a **property** that must hold for
**all** inputs, and the tool generates random cases (and "shrinks" a minimal
counterexample on failure). Two crates: **`proptest`** and **`quickcheck`**.

```toml
[dev-dependencies]
proptest = "1"
```

```rust
use proptest::prelude::*;

proptest! {
    // Property: clamp always stays within [1, 8], whatever the integer.
    #[test]
    fn clamp_stays_in_range(n in any::<usize>()) {
        let v = clamp_max_links(n);
        prop_assert!(v >= 1 && v <= 8);
    }
}
```

This is ideal for "pure" functions (parsing, scoring, normalization).

---

## 10. Testing a command-line interface (CLI)

To actually run the binary and check the exit code, stdout, and stderr, use
**`assert_cmd`** (+ **`predicates`** for flexible assertions)
[5](https://lobehub.com/skills/comeonoliver-skillshub-rust-testing).

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn lists_paths_by_complexity() {
    Command::cargo_bin("schema-pathfinder")
        .unwrap()
        .args([
            "path", "ClothingItem", "User",
            "--fixture", "../../fixtures/postgres/dressshot_seed_fk_edges.json",
            "--max-links", "5",
        ])
        .assert()
        .success()
        .stdout(contains("Path 1"));
}
```

This is the **E2E** level for a CLI: you test the product exactly as the user
invokes it.

---

## 11. Testing asynchronous code

The standard `#[test]` does not handle `async`. You use the runtime's test
macro:

```rust
#[tokio::test]
async fn fetches_data() {
    let value = load().await;
    assert_eq!(value, 42);
}
```

(or `#[async_std::test]`, `#[actix_rt::test]` depending on the runtime).

---

## 12. Mocking and test doubles

To isolate a unit from its dependencies (database, network…), you program
**against a trait** and inject a fake implementation. The **`mockall`** crate
generates the mocks:

```rust
use mockall::automock;

#[automock]
trait RepoClient {
    fn name(&self, id: u32) -> Option<String>;
}

#[test]
fn uses_the_repo() {
    let mut mock = MockRepoClient::new();
    mock.expect_name().returning(|_| Some("Alice".into()));
    assert_eq!(mock.name(1), Some("Alice".into()));
}
```

Key principle: "trait-first". This repository keeps the core **pure** (it works
on `&[ForeignKeyEdge]` in memory), which avoids the need for mocks in most of
the logic — the best way to avoid mocking is to design pure functions.

---

## 13. Code coverage (knowing what is NOT tested)

Coverage measures which lines/branches are executed by the tests.

### `cargo-llvm-cov` (recommended)

Based on compiler instrumentation, accurate, cross-platform, and it handles
doctests [3](https://lib.rs/crates/cargo-llvm-cov)
[2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

cargo llvm-cov                         # per-file summary
cargo llvm-cov --html                  # HTML report: target/llvm-cov/html/index.html
cargo llvm-cov --workspace             # the whole workspace
cargo llvm-cov --lcov --output-path lcov.info   # for CI (Codecov/Coveralls)
```

### `cargo-tarpaulin` (fast Linux alternative)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --engine llvm
```

Comparison: `cargo-llvm-cov` is the most accurate (branch coverage, doctests);
`tarpaulin` is easier to install but Linux-only and without branch coverage
[2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

> Aim for **branch** coverage, not only line coverage, but do not turn 100% into
> a dogma: some paths (hardware, network errors) are better tested via
> integration. Track exceptions in a list rather than cheating
> [2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

---

## 14. Running and filtering tests

```bash
cargo test                          # all package tests (unit + integration + doc)
cargo test -p schema-pathfinder     # a single workspace package
cargo test best_path                # only tests whose name contains "best_path"
cargo test --test path_search       # only the tests/path_search.rs file
cargo test --doc                    # only the doctests
cargo test -- --ignored             # only tests marked #[ignore]
cargo test -- --nocapture           # show the tests' println! output
cargo test -- --test-threads=1      # sequential execution (debugging)
```

Note: if the **unit** tests fail, Cargo does not run the following sections
(integration, doc) [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

### Going faster: `cargo-nextest`

```bash
cargo install cargo-nextest
cargo nextest run
```

A faster test runner with more readable output, compatible with CI.

---

## 15. Continuous integration (CI)

The real safety net is running all of this **on every push**. GitHub Actions
example:

```yaml
name: ci
on: [push, pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy, llvm-tools-preview }
      - uses: Swatinem/rust-cache@v2        # dependency cache
      - run: cargo fmt --all --check         # formatting
      - run: cargo clippy --all-targets -- -D warnings  # strict lint
      - run: cargo test --workspace          # all tests
      - run: cargo install cargo-llvm-cov && cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4      # upload coverage
        with: { files: lcov.info }

  node:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - run: corepack enable
      - run: pnpm install
      - run: pnpm test
      - run: pnpm lint
```

CI best practices: cache dependencies, `-D warnings` to reject any warning, and
a **matrix** (several OSes / Rust versions) if the project must stay portable
[5](https://lobehub.com/skills/comeonoliver-skillshub-rust-testing).

---

## 16. Concrete application to `schema-pathfinder`

This repository combines a Rust core and Node surfaces. Here is the complete
test map.

### Rust side (`apps/cli-rust`)

- **Unit tests** (in `src/pathfinder_core.rs`, `#[cfg(test)]` module): SQL
  parsing, rendering of a path, and private helpers (`score_path`,
  `resolve_table`, `build_adjacency`, `contains_table`, `quote_identifier`,
  `render_text`, `path_header`, `collect_paths`, `edge_constraint_signature`,
  `clamp_max_links`).
- **Integration tests** (folder `apps/cli-rust/tests/`), one file per feature,
  with helpers in `tests/common/mod.rs`:
  - `path_search.rs` — `best_path`, `ranked_paths_by_complexity`, `--max-links`
    limits, "no path" / "missing table" cases.
  - `scoring.rs` — score values, technical/auth table penalties.
  - `renderers.rs` — `render_path` (5 formats) and `render_paths` (multi-path).
  - `sql_metadata.rs` — DDL parsing (inline FK, table constraint, `ALTER TABLE`,
    non-`public` schemas, ignored statements, invalid input).
  - `fixture_loading.rs` — loading a JSON fixture / SQL file and errors.
  - `output_formats.rs` — `parse_format` / `format_name`.
  - `architecture_tree.rs` — ASCII tree rendering.
  - `database_url.rs` — rewriting the database name in the URL.

Commands:

```bash
cargo test -p schema-pathfinder            # core (lib + integration + doctests)
cargo test --workspace                     # everything (including the desktop)
cargo build --release -p schema-pathfinder --bin schema-pathfinder
```

### Node side (`packages/*`, `apps/web`, `apps/desktop`)

The Node tests (`node --test`) validate the JS mirror core and **inspect the
source code** of the surfaces (Rust, Slint, React) to lock down behaviors (e.g.
the presence of the "Max links" field, scrolling of the tables list). See
`tests/*.test.mjs`.

```bash
pnpm test         # all Node tests
pnpm typecheck    # scaffold check
pnpm lint         # scaffold + contract fixtures
```

---

## 17. "Complete tests" checklist

- [ ] Every public function has at least one integration test (happy path + at
      least one error path).
- [ ] Every non-trivial private helper has a unit test.
- [ ] Edge cases are covered (empty, min/max values, invalid input).
- [ ] The documentation examples are doctests that pass.
- [ ] "Combinatorial" pure functions have a property test.
- [ ] The CLI is tested end-to-end (`assert_cmd`).
- [ ] Coverage is measured and monitored (`cargo llvm-cov`).
- [ ] Everything runs in CI on every push, with `-D warnings` and a cache.
- [ ] Tests are fast, isolated, and deterministic (no implicit ordering, no
      shared I/O between tests).

---

## Sources

- The Rust Programming Language — Test Organization
  [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- Best Practices for Organizing Tests in Rust Projects
  [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/)
- Rust Testing Best Practices (organization, assert_cmd, mocking, CI)
  [5](https://lobehub.com/skills/comeonoliver-skillshub-rust-testing)
- cargo-llvm-cov (source-based coverage)
  [3](https://lib.rs/crates/cargo-llvm-cov)
- Rust Engineering Practices — Code Coverage
  [2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html)
