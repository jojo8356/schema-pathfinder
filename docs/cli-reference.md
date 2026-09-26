# Référence CLI

## Commande générale

```bash
schema-pathfinder <commande> [options]
```

Sources supportées :

- `--fixture <path>` : charge une fixture JSON.
- `--sql <path>` : parse un fichier DDL PostgreSQL.
- `--database-url <url>` : inspecte une base PostgreSQL.
- `DATABASE_URL` : fallback si aucune source explicite n'est donnée.

Priorité des sources : fixture, puis SQL, puis URL PostgreSQL, puis variable d'environnement.

## Aide

```bash
schema-pathfinder --help
```

Affiche les commandes, options et formats de sortie disponibles.

## Lister les tables

```bash
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder tables --sql fixtures/postgres/dressshot_schema.sql
schema-pathfinder --tables --database-url postgres://readonly:change-me@localhost:5432/postgres
```

La sortie utilise le format `schema.table` pour éviter les ambiguïtés. En mode PostgreSQL, les tables sont recherchées dans tous les schémas applicatifs visibles, et pas uniquement dans `public`.

## Lister les schémas

```bash
schema-pathfinder --schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Cette commande nécessite une source PostgreSQL réelle, pas une fixture ni un fichier SQL.

## Lister les bases

```bash
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder dbs --database-url postgres://readonly:change-me@localhost:5432/postgres
```

PostgreSQL ne donne pas toujours accès à toutes les bases. Le résultat dépend des droits du rôle utilisé et de la configuration serveur.

## Afficher l'arbre complet

```bash
schema-pathfinder --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Le mode tree liste les bases visibles, reconnecte à chaque base accessible, puis affiche les schémas, tables et relations sortantes.

## Trouver un chemin

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path public.ClothingItem public.User --format sql --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Formats disponibles :

- `text` : résumé lisible avec score.
- `equation` : équations de jointure, avec un saut de ligne à chaque `->`.
- `sql` : squelette de requête `JOIN` en lecture.
- `mermaid` : diagramme texte compatible Mermaid.
- `json` : contrat complet exploitable par une autre surface.

## Convertir du SQL en fixture

```bash
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

Cette commande permet de transformer un DDL PostgreSQL en fixture JSON et de réutiliser ensuite cette fixture sans base en ligne.

## Codes d'erreur

Les erreurs exposent un code stable en anglais, par exemple :

- `ARGS_INVALID`
- `DB_CONFIG_MISSING`
- `SOURCE_KIND_UNSUPPORTED`
- `PATH_NOT_FOUND`
- `FIXTURE_RENDER_FAILED`

Ces codes facilitent l'intégration dans des scripts ou une UI.
