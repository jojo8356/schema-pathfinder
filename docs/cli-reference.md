# Référence CLI

## Forme générale

```bash
schema-pathfinder <commande> [options]
```

Les options globales peuvent être placées avant ou après la commande :

```bash
schema-pathfinder path ClothingItem User --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json path ClothingItem User
```

## Commandes

| Commande                     | Rôle                                                              | Source              |
| ---------------------------- | ----------------------------------------------------------------- | ------------------- |
| `tables`                     | liste les tables, une `schema.table` par ligne                     | fixture, SQL, PG    |
| `schemas`                    | liste les schémas applicatifs d'une base                           | PostgreSQL          |
| `databases` (alias `dbs`)    | liste les bases visibles et connectables                           | PostgreSQL          |
| `tree`                       | affiche bases → schémas → tables → FK sortantes                    | PostgreSQL          |
| `path <source> <cible>`      | liste tous les chemins FK entre deux tables, du plus simple au plus complexe | fixture, SQL, PG |
| `fixture`                    | convertit la source choisie en fixture JSON                        | fixture, SQL, PG    |
| `help [commande]`            | affiche l'aide générale ou l'aide d'une commande                   | —                   |
| `version`                    | affiche la version                                                 | —                   |

Chaque commande qui liste des éléments affiche un marqueur explicite quand le
résultat est vide : `NO_TABLES`, `NO_SCHEMAS`, `NO_DATABASES`. Les scripts peuvent
donc distinguer « vide » d'une erreur sans analyser la sortie ligne à ligne.

## Options globales

| Option                 | Rôle                                                                     |
| ---------------------- | ------------------------------------------------------------------------ |
| `--fixture <path>`     | lit les métadonnées FK depuis une fixture JSON                            |
| `--sql <path>`         | lit un DDL PostgreSQL et le convertit en métadonnées                      |
| `--database-url <url>` | lit les métadonnées depuis cette URL de connexion PostgreSQL              |
| `--max-links <n>`      | nombre maximal de liens FK par chemin (`path`), défaut `5`, borné de 1 à 8 |
| `--format <fmt>`       | format de sortie (`path`) : `text`, `equation`, `json`, `sql`, `mermaid`   |
| `-h`, `--help`         | affiche l'aide (générale, ou celle de la commande si elle est présente)   |
| `-V`, `--version`      | affiche la version                                                        |

Une seule source explicite peut être fournie à la fois : combiner `--fixture`,
`--sql` et `--database-url` est une erreur d'usage. Sans source explicite, la
variable d'environnement `DATABASE_URL` est utilisée.

`--format` ne s'applique qu'à `path` : les commandes de listing affichent une
entrée par ligne pour rester composables avec `grep`, `sort` ou une boucle shell.

## Aide et version

```bash
schema-pathfinder --help          # aide générale
schema-pathfinder help            # identique
schema-pathfinder help path       # aide de la commande path
schema-pathfinder tables --help   # identique à "help tables", la commande n'est pas exécutée
schema-pathfinder version         # ou --version / -V
```

## Lister les tables

```bash
schema-pathfinder tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder tables --sql fixtures/postgres/dressshot_schema.sql
schema-pathfinder tables --database-url postgres://readonly:change-me@localhost:5432/postgres
```

La sortie utilise le format `schema.table` pour éviter les ambiguïtés, dans un
ordre trié et donc stable d'un appel à l'autre. En mode PostgreSQL, les tables
sont recherchées dans tous les schémas applicatifs visibles, pas uniquement dans
`public`.

## Lister les schémas

```bash
schema-pathfinder schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Cette commande nécessite une source PostgreSQL réelle : avec `--fixture` ou
`--sql`, elle renvoie `SOURCE_KIND_UNSUPPORTED`. Les schémas `information_schema`
et `pg_*` sont exclus.

## Lister les bases

```bash
schema-pathfinder databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder dbs --database-url postgres://readonly:change-me@localhost:5432/postgres
```

PostgreSQL ne donne pas toujours accès à toutes les bases : le résultat dépend
des droits du rôle utilisé et de la configuration serveur. Les bases modèles et
les bases qui interdisent les connexions sont exclues.

## Afficher l'arbre complet

```bash
schema-pathfinder tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Le mode tree liste les bases visibles, se reconnecte à chaque base accessible,
puis affiche les schémas, tables et relations sortantes. Une base inaccessible
reste listée avec son erreur (`error DB_CONNECT_FAILED: ...`) au lieu de faire
échouer toute la commande.

```text
postgres
|-- database app
|  `-- schema public
|     |-- table ClothingItem
|     |  `-- fk clothingSessionId -> public.ClothingSession.id (ClothingItem_clothingSessionId_fkey)
|     `-- table ClothingSession
`-- database postgres
```

## Trouver un chemin

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path public.ClothingItem public.User --format sql --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder path BON_DE_LIVRAISON COMMANDE_CLIENT --max-links 3 --database-url postgres://readonly:change-me@localhost:5432/postgres
```

La commande `path` liste **tous** les chemins de clés étrangères entre les deux
tables, du plus simple au plus complexe : d'abord les chemins à 1 lien, puis ceux
à 2 liens, etc. À longueur égale, le chemin au meilleur score est affiché en
premier, puis l'ordre est déterministe (signature des contraintes). Chaque chemin
est numéroté (`Path 1`, `Path 2`, ...) et le nombre de chemins renvoyés est
plafonné à 50.

C'est utile quand le chemin le plus « évident » passe par une table qui n'est pas
encore renseignée au moment de la saisie : on peut alors choisir un autre chemin
plus court ou qui évite cette table.

- `--max-links <n>` : nombre maximal de liens par chemin. Par défaut `5`, borné
  entre `1` et `8`. Une valeur hors bornes est ramenée dans l'intervalle ; une
  valeur non numérique est une erreur d'usage. Au-delà de 5 liens, les jointures
  deviennent en général trop complexes à saisir.

Formats disponibles :

- `text` : résumé lisible avec score.
- `equation` : équations de jointure, avec un saut de ligne à chaque `->`.
- `sql` : squelette de requête `JOIN` en lecture seule.
- `mermaid` : diagramme texte compatible Mermaid.
- `json` : contrat complet exploitable par une autre surface.

## Convertir du SQL en fixture

```bash
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

Cette commande permet de transformer un DDL PostgreSQL en fixture JSON et de
réutiliser ensuite cette fixture sans base en ligne.

## Compatibilité

Les drapeaux historiques restent acceptés et sont équivalents aux sous-commandes :

```bash
schema-pathfinder --tables    --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder --schemas   --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --tree      --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Ils ne peuvent pas être combinés entre eux, ni avec une sous-commande.

## Codes de sortie

| Code | Signification                                                                 |
| ---- | ----------------------------------------------------------------------------- |
| `0`  | succès                                                                         |
| `1`  | erreur d'exécution (fichier absent, base injoignable, aucun chemin, ...)       |
| `2`  | erreur d'usage (argument invalide, commande inconnue, format non supporté)     |

## Codes d'erreur

Les erreurs sont écrites sur stderr sous la forme `CODE: message`, avec un code
stable en anglais :

- `ARGS_INVALID` : argument, commande ou nombre d'arguments invalide (sortie 2).
- `UNSUPPORTED_FORMAT` : format de sortie inconnu (sortie 2).
- `DB_CONFIG_MISSING` : aucune source et pas de `DATABASE_URL`.
- `SOURCE_KIND_UNSUPPORTED` : commande PostgreSQL appelée avec une fixture ou un SQL.
- `DB_CONNECT_FAILED`, `DB_METADATA_FAILED` : connexion ou introspection en échec.
- `FIXTURE_READ_FAILED`, `FIXTURE_JSON_INVALID`, `FIXTURE_RENDER_FAILED` : fixture illisible.
- `SQL_READ_FAILED`, `SQL_PARSE_FAILED` : DDL illisible ou non parsable.
- `TABLE_NOT_FOUND` : table source ou cible absente des métadonnées.
- `NO_DECLARED_FK_PATH` : aucun chemin FK déclaré entre les deux tables.

Ces codes facilitent l'intégration dans des scripts ou une UI.
