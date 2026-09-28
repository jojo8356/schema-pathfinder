# Vue Employeur

## Résumé

Schema Pathfinder est un projet indépendant d'outillage base de données. Il aide à comprendre rapidement les relations entre tables PostgreSQL en exploitant les clés étrangères déclarées. Le projet est construit comme un vrai produit technique : CLI, UI desktop, UI web, API, fixtures de contrat, tests, packaging et documentation.

## Problème traité

Sur une base existante, comprendre comment deux tables sont liées peut prendre du temps : il faut inspecter les schémas, les contraintes, les tables intermédiaires et parfois plusieurs bases. Schema Pathfinder automatise cette recherche et fournit un résultat lisible dans plusieurs formats.

## Valeur technique

- Lecture de métadonnées PostgreSQL sans requêter les données métier.
- Recherche de chemins dans un graphe de clés étrangères.
- Rendu adapté à plusieurs usages : humain, SQL, diagramme, JSON.
- Mode offline via fixture ou fichier SQL.
- Mode connecté via URL PostgreSQL.
- Distribution en binaires et paquets installables.

## Points observables dans le dépôt

- Architecture multi-surfaces cohérente : le coeur Rust est partagé par le CLI, l'API et le desktop.
- API séparée uniquement pour le web, afin de garder le CLI et le desktop autonomes.
- Fixtures JSON pour stabiliser les contrats entre surfaces.
- Tests automatisés sur les comportements critiques.
- Documentation claire pour prise en main, audit et contribution.

## Périmètre volontaire

Le MVP reste centré sur les clés étrangères déclarées. Il ne fait pas encore d'inférence sémantique sur les noms de colonnes, de scoring statistique sur les données, ni de lecture des lignes métier. Ce choix rend l'outil plus sûr, plus prévisible et plus simple à vérifier.

## Comment évaluer rapidement le projet

```bash
pnpm test
cargo test -p schema-pathfinder
cargo run -p schema-pathfinder --bin schema-pathfinder -- --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Si aucune base locale n'est disponible, la fixture incluse suffit :

```bash
cargo run -p schema-pathfinder --bin schema-pathfinder -- path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
```
