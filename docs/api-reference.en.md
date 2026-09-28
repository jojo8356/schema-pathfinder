# API Reference

The Rust API serves the web UI and exposes the same main capabilities as the core. It is separate from the CLI and desktop: both of those surfaces remain usable without a server.

## Startup

```bash
cargo run -p schema-pathfinder --features api --bin schema-pathfinder-api
```

Useful variables:

- `SCHEMA_PATHFINDER_HOST`: listen address.
- `SCHEMA_PATHFINDER_PORT`: listen port.
- `DATABASE_URL`: default PostgreSQL source.
- `SCHEMA_PATHFINDER_WEB_DIR`: static folder of the built web UI.
- `SCHEMA_PATHFINDER_API_TOKEN`: optional token to protect the API.

## Health

```http
GET /api/health
```

Expected response:

```json
{ "status": "ok" }
```

## Tables

```http
GET /api/tables
POST /api/tables
```

The `POST` accepts an explicit source: fixture, SQL, or PostgreSQL URL.

Example:

```json
{
  "sourceKind": "postgres",
  "sourceValue": "postgres://readonly:change-me@localhost:5432/postgres"
}
```

## Databases

```http
POST /api/databases
```

Returns the databases visible from the provided PostgreSQL URL. The result depends on the role's privileges.

## Schemas

```http
POST /api/schemas
```

Returns the non-system schemas visible in the target database.

## Path

```http
POST /api/path
```

Example:

```json
{
  "sourceKind": "fixture",
  "sourceValue": "fixtures/postgres/dressshot_seed_fk_edges.json",
  "sourceTable": "ClothingItem",
  "targetTable": "User",
  "format": "equation"
}
```

## Errors

Errors follow a stable shape:

```json
{
  "error": {
    "code": "PATH_NOT_FOUND",
    "title": "Path not found"
  }
}
```

The title stays technical and in English so backend logic and final UI text are not mixed together.
