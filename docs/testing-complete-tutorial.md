# Tutoriel complet : écrire tous les tests d'un projet

Ce guide explique, étape par étape, comment couvrir un projet avec **tous les
types de tests** utiles. Il est écrit pour ce dépôt (`schema-pathfinder`, un
cœur Rust + des surfaces Node/React), mais les principes s'appliquent à
n'importe quel projet.

L'objectif : passer d'un projet « ça marche sur ma machine » à un projet dont le
comportement est **vérifié automatiquement**, à chaque commit, du plus petit
détail (une fonction privée) jusqu'au parcours utilisateur complet.

---

## 1. La philosophie : la pyramide des tests

On distingue trois grandes familles, du plus petit au plus large :

1. **Tests unitaires** — vérifient une fonction / un module **isolé**. Rapides,
   nombreux, accèdent au code privé.
2. **Tests d'intégration** — vérifient que plusieurs parties fonctionnent
   ensemble via l'**API publique**. Moins nombreux, plus lents.
3. **Tests de bout en bout (E2E)** — vérifient le produit complet (CLI lancée
   pour de vrai, requête HTTP réelle, UI dans un navigateur). Peu nombreux, lents.

Règle pratique : beaucoup d'unitaires, quelques intégrations, très peu d'E2E.
On teste **le comportement observable**, pas les détails d'implémentation
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 2. Les trois types de tests en Rust

Rust supporte nativement **trois** sortes de tests : unitaires, d'intégration et
de documentation (doctests) [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

| Type | Emplacement | Accès | Compilé quand |
| --- | --- | --- | --- |
| Unitaire | dans `src/*.rs`, module `#[cfg(test)]` | code **privé** inclus | `cargo test` |
| Intégration | dossier `tests/` à la racine du crate | API **publique** seulement | `cargo test` |
| Doctest | dans les commentaires `///` | API publique | `cargo test` |

---

## 3. Tests unitaires

Ils vivent **dans le même fichier** que le code testé, dans un module `tests`
annoté `#[cfg(test)]`. L'attribut `#[cfg(test)]` fait que ce code n'est compilé
et lié **que** pendant `cargo test` : il ne pèse pas dans le binaire de release
et ses dépendances de test ne fuient pas en production
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*; // importe le contenu du module parent, y compris le privé

    #[test]
    fn additionne_deux_nombres() {
        assert_eq!(add(2, 2), 4);
    }
}
```

### Tester des fonctions privées

C'est **le** super-pouvoir des tests unitaires : comme ils sont dans le module,
`use super::*;` donne accès aux fonctions privées. Dans ce dépôt,
`apps/cli-rust/src/pathfinder_core.rs` teste ainsi des helpers privés
(`score_path`, `resolve_table`, `build_adjacency`, `quote_identifier`,
`clamp_max_links`, …) :

```rust
#[test]
fn quote_identifier_doubles_embedded_quotes() {
    assert_eq!(quote_identifier("a\"b"), "\"a\"\"b\"");
}
```

Gardez ces tests **rapides et sans I/O**. « Si ça touche le système de fichiers,
ce n'est pas un test unitaire »
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 4. Tests d'intégration

Ils vivent dans un dossier **`tests/`** placé à la racine du crate (à côté de
`src/` et `Cargo.toml`). **Chaque fichier `tests/*.rs` est compilé comme une
crate séparée** qui dépend de votre bibliothèque comme le ferait un utilisateur
externe : ils ne voient donc que l'**API publique**
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

Si vous tentez d'appeler une fonction privée depuis `tests/`, le compilateur
refuse (erreur `E0603`). C'est **voulu** : ça force à tester l'interface que vos
utilisateurs consomment réellement
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

### Astuce binaire vs bibliothèque

Un crate **binaire** (`main.rs` seul) n'expose rien : on ne peut pas l'importer
depuis `tests/`. La bonne pratique est de mettre la logique dans `src/lib.rs` et
de garder un `main.rs` minimal qui l'appelle
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html). C'est
exactement ce que fait ce dépôt : toute la logique est dans
`schema_pathfinder::pathfinder_core`, et `main.rs` / le binaire API ne font que
l'orchestrer.

### Code partagé entre fichiers de tests : `tests/common/mod.rs`

Comme chaque fichier est une crate distincte, on ne peut pas simplement importer
une fonction d'un autre fichier de test. La convention est de créer
**`tests/common/mod.rs`** (et non `tests/common.rs`, qui serait vu comme un
test) et de faire `mod common;` dans chaque fichier
[1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

```rust
// tests/common/mod.rs
#![allow(dead_code)] // certains helpers ne servent pas dans tous les fichiers

pub fn table(name: &str) -> TableIdentifier { /* ... */ }
pub fn write_temp_file(prefix: &str, content: &str) -> PathBuf { /* ... */ }
```

```rust
// tests/fixture_loading.rs
mod common;
use common::write_temp_file;
```

> ⚠️ Piège classique : `src/tests/` **n'est pas** un dossier de tests
> d'intégration. Cargo ne regarde que `tests/` à la racine du crate. Un
> `src/tests/` est traité comme un simple module et vos tests ne s'exécutent pas
> [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

### Nommer les fichiers par fonctionnalité

Nommez `tests/path_search.rs`, `tests/renderers.rs`, `tests/sql_metadata.rs` …
d'après le **comportement** testé, pas d'après un module interne
[3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 5. Doctests (tests dans la documentation)

Tout bloc de code dans un commentaire `///` est compilé **et exécuté** par
`cargo test`. C'est parfait pour garantir que les exemples de la doc restent
justes.

```rust
/// Normalise une valeur de « max links » dans l'intervalle supporté.
///
/// ```
/// use schema_pathfinder::pathfinder_core::clamp_max_links;
/// assert_eq!(clamp_max_links(0), 1);
/// assert_eq!(clamp_max_links(99), 8);
/// ```
pub fn clamp_max_links(max_links: usize) -> usize { /* ... */ }
```

Astuces :
- ` ```no_run ` compile mais n'exécute pas (utile pour du code réseau/DB).
- ` ```ignore ` ni compilé ni exécuté.
- ` ```should_panic ` vérifie que l'exemple panique.
- Une ligne préfixée de `# ` est masquée dans la doc rendue mais présente à la
  compilation.

---

## 6. Assertions et tests d'erreurs

### Macros d'assertion

```rust
assert!(condition, "message optionnel {}", valeur);
assert_eq!(gauche, droite);
assert_ne!(gauche, droite);
```

### Tester qu'une fonction panique

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn panique_hors_limites() {
    let v = vec![1, 2, 3];
    let _ = v[10];
}
```

### Tester les chemins d'erreur (recommandé pour les `Result`)

Plutôt que de « unwrap » partout, vérifiez explicitement le code d'erreur :

```rust
#[test]
fn rejette_un_format_inconnu() {
    let error = parse_format("yaml").expect_err("devrait échouer");
    assert_eq!(error.code, "UNSUPPORTED_FORMAT");
}
```

### Tests qui renvoient `Result`

Une fonction de test peut renvoyer `Result<(), E>` : on peut alors utiliser `?`.

```rust
#[test]
fn charge_une_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let schema = load_schema_from_fixture("fixtures/x.json")?;
    assert!(!schema.edges.is_empty());
    Ok(())
}
```

### Ignorer temporairement un test

```rust
#[test]
#[ignore = "nécessite une base PostgreSQL locale"]
fn interroge_postgres() { /* ... */ }
```

On le lance ensuite avec `cargo test -- --ignored`.

---

## 7. Fixtures, setup et données de test

- **Constantes / builders** : préférez de petits *builders* (`fn table(...)`,
  `fn fk(...)`) plutôt que de dupliquer des littéraux. Voir
  `apps/cli-rust/tests/common/mod.rs`.
- **Fichiers temporaires** : écrivez dans `std::env::temp_dir()` avec un nom
  unique (PID + compteur atomique) pour rester **indépendant du répertoire
  courant** (Cargo lance les tests depuis la racine du crate) et **thread-safe**
  (les tests tournent en parallèle par défaut).
- **Setup/teardown** : Rust n'a pas de `beforeEach` natif ; on appelle une
  fonction d'initialisation partagée depuis `tests/common/mod.rs`
  [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/).

---

## 8. Tests paramétrés avec `rstest`

Pour exécuter le même test sur plusieurs jeux de valeurs sans copier-coller,
la crate **`rstest`** est la référence.

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
fn clamp_borne_les_valeurs(#[case] entree: usize, #[case] attendu: usize) {
    assert_eq!(clamp_max_links(entree), attendu);
}
```

`rstest` gère aussi les **fixtures** (fonctions `#[fixture]` injectées en
argument).

---

## 9. Tests basés sur les propriétés (property-based)

Au lieu de choisir des exemples à la main, on décrit une **propriété** qui doit
être vraie pour **toutes** les entrées, et l'outil génère des cas aléatoires
(et « rétrécit » un contre-exemple minimal en cas d'échec). Deux crates :
**`proptest`** et **`quickcheck`**.

```toml
[dev-dependencies]
proptest = "1"
```

```rust
use proptest::prelude::*;

proptest! {
    // Propriété : clamp reste toujours dans [1, 8], quel que soit l'entier.
    #[test]
    fn clamp_reste_dans_l_intervalle(n in any::<usize>()) {
        let v = clamp_max_links(n);
        prop_assert!(v >= 1 && v <= 8);
    }
}
```

C'est idéal pour les fonctions « pures » (parsing, scoring, normalisation).

---

## 10. Tester une interface en ligne de commande (CLI)

Pour lancer réellement le binaire et vérifier code de sortie, stdout et stderr,
on utilise **`assert_cmd`** (+ **`predicates`** pour les assertions souples)
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
fn liste_les_chemins_par_complexite() {
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

C'est le niveau **E2E** pour une CLI : on teste le produit tel que l'utilisateur
l'invoque.

---

## 11. Tester du code asynchrone

Le `#[test]` standard ne gère pas `async`. On utilise la macro de test du
runtime :

```rust
#[tokio::test]
async fn recupere_les_donnees() {
    let valeur = charge().await;
    assert_eq!(valeur, 42);
}
```

(ou `#[async_std::test]`, `#[actix_rt::test]` selon le runtime).

---

## 12. Mocking et test doubles

Pour isoler une unité de ses dépendances (base de données, réseau…), on
programme **contre un trait** puis on injecte une implémentation factice. La
crate **`mockall`** génère les mocks :

```rust
use mockall::automock;

#[automock]
trait DepotClient {
    fn nom(&self, id: u32) -> Option<String>;
}

#[test]
fn utilise_le_depot() {
    let mut mock = MockDepotClient::new();
    mock.expect_nom().returning(|_| Some("Alice".into()));
    assert_eq!(mock.nom(1), Some("Alice".into()));
}
```

Principe clé : « trait-first ». Ce dépôt garde le cœur **pur** (il travaille sur
des `&[ForeignKeyEdge]` en mémoire), ce qui évite le besoin de mocks pour la
majorité de la logique — la meilleure façon d'éviter le mocking est de
concevoir des fonctions pures.

---

## 13. Couverture de code (savoir ce qui n'est PAS testé)

La couverture mesure quelles lignes/branches sont exécutées par les tests.

### `cargo-llvm-cov` (recommandé)

Basé sur l'instrumentation du compilateur, précis, multiplateforme, gère les
doctests [3](https://lib.rs/crates/cargo-llvm-cov)
[2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

cargo llvm-cov                         # résumé par fichier
cargo llvm-cov --html                  # rapport HTML : target/llvm-cov/html/index.html
cargo llvm-cov --workspace             # tout l'espace de travail
cargo llvm-cov --lcov --output-path lcov.info   # pour la CI (Codecov/Coveralls)
```

### `cargo-tarpaulin` (alternative rapide sous Linux)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --engine llvm
```

Comparatif : `cargo-llvm-cov` est le plus précis (couverture de branches,
doctests) ; `tarpaulin` est plus simple à installer mais Linux uniquement et
sans couverture de branches
[2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

> Visez la couverture **de branches** et pas seulement de lignes, mais ne
> transformez pas 100 % en dogme : certains chemins (matériel, erreurs réseau)
> se testent mieux en intégration. Suivez les exceptions dans une liste plutôt
> que de tricher [2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html).

---

## 14. Lancer et filtrer les tests

```bash
cargo test                          # tous les tests du paquet (unit + intégration + doc)
cargo test -p schema-pathfinder     # un seul paquet de l'espace de travail
cargo test best_path                # seulement les tests dont le nom contient "best_path"
cargo test --test path_search       # seulement le fichier tests/path_search.rs
cargo test --doc                    # seulement les doctests
cargo test -- --ignored             # seulement les tests marqués #[ignore]
cargo test -- --nocapture           # affiche les println! des tests
cargo test -- --test-threads=1      # exécution séquentielle (débogage)
```

À noter : si les tests **unitaires** échouent, Cargo n'exécute pas les sections
suivantes (intégration, doc) [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

### Aller plus vite : `cargo-nextest`

```bash
cargo install cargo-nextest
cargo nextest run
```

Un lanceur de tests plus rapide et à la sortie plus lisible, compatible CI.

---

## 15. Intégration continue (CI)

Le vrai filet de sécurité, c'est d'exécuter tout ça **à chaque push**. Exemple
GitHub Actions :

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
      - uses: Swatinem/rust-cache@v2        # cache des dépendances
      - run: cargo fmt --all --check         # format
      - run: cargo clippy --all-targets -- -D warnings  # lint strict
      - run: cargo test --workspace          # tous les tests
      - run: cargo install cargo-llvm-cov && cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4      # upload couverture
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

Bonnes pratiques CI : mise en cache des dépendances, `-D warnings` pour refuser
tout avertissement, et une **matrice** (plusieurs OS / versions de Rust) si le
projet doit rester portable [5](https://lobehub.com/skills/comeonoliver-skillshub-rust-testing).

---

## 16. Application concrète à `schema-pathfinder`

Ce dépôt combine un cœur Rust et des surfaces Node. Voici la carte complète des
tests.

### Côté Rust (`apps/cli-rust`)

- **Tests unitaires** (dans `src/pathfinder_core.rs`, module `#[cfg(test)]`) :
  parsing SQL, rendu d'un chemin, et helpers privés (`score_path`,
  `resolve_table`, `build_adjacency`, `contains_table`, `quote_identifier`,
  `render_text`, `path_header`, `collect_paths`, `edge_constraint_signature`,
  `clamp_max_links`).
- **Tests d'intégration** (dossier `apps/cli-rust/tests/`), un fichier par
  fonctionnalité, avec des helpers dans `tests/common/mod.rs` :
  - `path_search.rs` — `best_path`, `ranked_paths_by_complexity`, limites
    `--max-links`, cas « pas de chemin » / « table absente ».
  - `scoring.rs` — valeurs de score, pénalités tables techniques / auth.
  - `renderers.rs` — `render_path` (5 formats) et `render_paths` (multi-chemins).
  - `sql_metadata.rs` — parsing DDL (FK inline, contrainte de table, `ALTER
    TABLE`, schémas non-`public`, statements ignorés, entrée invalide).
  - `fixture_loading.rs` — chargement fixture JSON / fichier SQL et erreurs.
  - `output_formats.rs` — `parse_format` / `format_name`.
  - `architecture_tree.rs` — rendu de l'arbre ASCII.
  - `database_url.rs` — réécriture du nom de base dans l'URL.

Commandes :

```bash
cargo test -p schema-pathfinder            # cœur (lib + intégration + doctests)
cargo test --workspace                     # tout (y compris le desktop)
cargo build --release -p schema-pathfinder --bin schema-pathfinder
```

### Côté Node (`packages/*`, `apps/web`, `apps/desktop`)

Les tests Node (`node --test`) valident le cœur JS miroir et **inspectent le
code source** des surfaces (Rust, Slint, React) pour verrouiller des
comportements (ex. présence du champ « Max links », défilement de la liste des
tables). Voir `tests/*.test.mjs`.

```bash
pnpm test         # tous les tests Node
pnpm typecheck    # vérification du scaffold
pnpm lint         # scaffold + fixtures de contrat
```

---

## 17. Checklist « tests complets »

- [ ] Chaque fonction publique a au moins un test d'intégration (chemin
      nominal + au moins un chemin d'erreur).
- [ ] Chaque helper privé non trivial a un test unitaire.
- [ ] Les cas limites sont couverts (vide, valeurs min/max, entrée invalide).
- [ ] Les exemples de la documentation sont des doctests qui passent.
- [ ] Les fonctions pures « à combinatoire » ont un test de propriété.
- [ ] La CLI est testée de bout en bout (`assert_cmd`).
- [ ] La couverture est mesurée et surveillée (`cargo llvm-cov`).
- [ ] Tout tourne en CI à chaque push, avec `-D warnings` et cache.
- [ ] Les tests sont rapides, isolés et déterministes (pas d'ordre implicite,
      pas d'I/O partagée entre tests).

---

## Sources

- The Rust Programming Language — Test Organization
  [1](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- Best Practices for Organizing Tests in Rust Projects
  [3](https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/)
- Rust Testing Best Practices (organisation, assert_cmd, mocking, CI)
  [5](https://lobehub.com/skills/comeonoliver-skillshub-rust-testing)
- cargo-llvm-cov (couverture source-based)
  [3](https://lib.rs/crates/cargo-llvm-cov)
- Rust Engineering Practices — Code Coverage
  [2](https://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html)
