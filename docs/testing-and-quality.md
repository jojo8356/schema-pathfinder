# Tests et Qualité

## Commandes principales

```bash
pnpm test
pnpm typecheck
pnpm lint
cargo test -p schema-pathfinder
cargo build --release -p schema-pathfinder --bin schema-pathfinder
cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api
```

## Ce qui est testé

- Contrats de fixtures JSON.
- Rendus de chemins en formats texte, equation, SQL, Mermaid et JSON.
- Parsing de SQL PostgreSQL vers métadonnées.
- Commandes CLI principales.
- Scaffolding attendu du monorepo.
- Comportements Rust du coeur pathfinder.

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
