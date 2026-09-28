# Référence API

L'API Rust sert l'UI web et expose les mêmes capacités principales que le coeur. Elle est séparée du CLI et du desktop : ces deux surfaces restent utilisables sans serveur.

## Lancement

```bash
cargo run -p schema-pathfinder --features api --bin schema-pathfinder-api
```

Variables utiles :

- `SCHEMA_PATHFINDER_HOST` : adresse d'écoute.
- `SCHEMA_PATHFINDER_PORT` : port d'écoute.
- `DATABASE_URL` : source PostgreSQL par défaut.
- `SCHEMA_PATHFINDER_WEB_DIR` : dossier statique de l'UI web buildée.
- `SCHEMA_PATHFINDER_API_TOKEN` : token optionnel pour protéger l'API.

## Santé

```http
GET /api/health
```

Réponse attendue :

```json
{ "status": "ok" }
```

## Tables

```http
GET /api/tables
POST /api/tables
```

Le `POST` accepte une source explicite : fixture, SQL ou URL PostgreSQL.

Exemple :

```json
{
  "sourceKind": "postgres",
  "sourceValue": "postgres://readonly:change-me@localhost:5432/postgres"
}
```

## Bases

```http
POST /api/databases
```

Retourne les bases visibles depuis l'URL PostgreSQL fournie. Le résultat dépend des privilèges du rôle.

## Schémas

```http
POST /api/schemas
```

Retourne les schémas non système visibles dans la base ciblée.

## Chemin

```http
POST /api/path
```

Exemple :

```json
{
  "sourceKind": "fixture",
  "sourceValue": "fixtures/postgres/dressshot_seed_fk_edges.json",
  "sourceTable": "ClothingItem",
  "targetTable": "User",
  "format": "equation"
}
```

## Erreurs

Les erreurs suivent une forme stable :

```json
{
  "error": {
    "code": "PATH_NOT_FOUND",
    "title": "Path not found"
  }
}
```

Le titre reste technique et en anglais afin de ne pas mélanger logique backend et texte final d'interface.
