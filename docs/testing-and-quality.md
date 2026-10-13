# Tests et Qualité

## Commandes principales

```bash
pnpm test                    # 78 tests Node
pnpm typecheck
pnpm lint
cargo test --workspace       # 152 tests Rust (unitaires, intégration, doctests)
cargo build --release -p schema-pathfinder --bin schema-pathfinder
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## Ce qui est testé

- Contrats de fixtures JSON.
- Rendus de chemins en formats texte, equation, SQL, Mermaid et JSON.
- Parsing de SQL PostgreSQL vers métadonnées : 33 tests d'intégration
  (`apps/cli-rust/tests/sql_metadata.rs`) et 12 tests unitaires blancs dans
  `postgres_sql_metadata.rs` (découpage des statements, commentaires, corps
  dollar-quotés, clés composées, `DROP CONSTRAINT`).
- Surface CLI : parsing pur dans `src/cli.rs` (23 tests unitaires) et 24 tests de
  bout en bout qui exécutent le binaire compilé via `CARGO_BIN_EXE_schema-pathfinder`
  et vérifient stdout, stderr et le code de sortie (`0`, `1`, `2`).
- Contrat de l'UI desktop (`tests/desktop_ui.test.mjs`) : structure responsive,
  absence de taille figée, et interdiction de lier une taille horizontale, un
  padding ou un `font-size` à `root.width` (boucle de binding Slint).
- Scaffolding attendu du monorepo.
- Comportements Rust du coeur pathfinder.

## Portes de qualité en CI

Le job `Node and Rust tests` de `.github/workflows/build-binaries.yml` exécute, dans l'ordre :
le contrat UI desktop, les tests Node, `pnpm lint`, `cargo test --workspace`, une vérification
d'absence de `binding loop` Slint sur l'UI desktop, puis le build du binaire API. Le job de
packaging (`.deb`, AppImage) dépend de ce job : aucun paquet n'est produit si les tests échouent.

## Stratégie qualité

Le projet privilégie des tests proches des contrats utilisateur : entrée source, chemin attendu, rendu attendu. Cette approche protège les surfaces CLI, web et desktop contre les divergences de format.

## Vérification manuelle utile

Pour une base PostgreSQL réelle :

```bash
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Pour une fixture locale :

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```

## Critères avant publication

- Tous les tests passent.
- Les binaires release compilent.
- Les docs ne contiennent aucun secret réel.
- Les artefacts générés ne polluent pas le commit sauf publication volontaire.
- Le README reste suffisant pour évaluer le projet en moins de quelques minutes.
