# CLI Reference

## General command

```bash
schema-pathfinder <command> [options]
```

Supported sources:

- `--fixture <path>`: load a JSON fixture.
- `--sql <path>`: parse a PostgreSQL DDL file.
- `--database-url <url>`: inspect a PostgreSQL database.
- `DATABASE_URL`: fallback when no explicit source is given.

Source priority: fixture, then SQL, then PostgreSQL URL, then environment variable.

## Help

```bash
schema-pathfinder --help
```

Displays the available commands, options, and output formats.

## List tables

```bash
schema-pathfinder --tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder tables --sql fixtures/postgres/dressshot_schema.sql
schema-pathfinder --tables --database-url postgres://readonly:change-me@localhost:5432/postgres
```

The output uses the `schema.table` format to avoid ambiguity. In PostgreSQL mode, tables are searched across all visible application schemas, not only `public`.

## List schemas

```bash
schema-pathfinder --schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
```

This command requires a real PostgreSQL source, not a fixture or SQL file.

## List databases

```bash
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder dbs --database-url postgres://readonly:change-me@localhost:5432/postgres
```

PostgreSQL does not always grant access to every database. The result depends on the privileges of the role used and on the server configuration.

## Show the full tree

```bash
schema-pathfinder --tree --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Tree mode lists the visible databases, reconnects to each accessible database, then displays the schemas, tables, and outgoing relationships.

## Find a path

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path public.ClothingItem public.User --format sql --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder path BON_DE_LIVRAISON COMMANDE_CLIENT --max-links 3 --database-url postgres://readonly:change-me@localhost:5432/postgres
```

The `path` command lists **all** foreign-key paths between the two tables, from
the simplest to the most complex: first the 1-link paths, then the 2-link ones,
and so on. For equal lengths, the highest-scoring path is shown first. Each path
is numbered (`Path 1`, `Path 2`, ...).

This is useful when the most "obvious" path goes through a table that is not yet
populated at data-entry time: you can then pick another path that is shorter or
that avoids that table.

- `--max-links <n>`: maximum number of links per path. Defaults to `5`, bounded
  between `1` and `8`. Beyond 5 links, joins usually become too complex to write
  by hand.

Available formats:

- `text`: readable summary with score.
- `equation`: join equations, with a line break at each `->`.
- `sql`: read-only `JOIN` query skeleton.
- `mermaid`: Mermaid-compatible text diagram.
- `json`: full contract consumable by another surface.

## Convert SQL to a fixture

```bash
schema-pathfinder fixture --sql fixtures/postgres/dressshot_schema.sql
```

This command turns a PostgreSQL DDL into a JSON fixture, which can then be reused without a live database.

## Error codes

Errors expose a stable code in English, for example:

- `ARGS_INVALID`
- `DB_CONFIG_MISSING`
- `SOURCE_KIND_UNSUPPORTED`
- `PATH_NOT_FOUND`
- `FIXTURE_RENDER_FAILED`

These codes make integration into scripts or a UI easier.
