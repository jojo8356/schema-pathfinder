# CLI Reference

## General form

```bash
schema-pathfinder <command> [options]
```

Global options may be placed before or after the command:

```bash
schema-pathfinder path ClothingItem User --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder --format sql --fixture fixtures/postgres/dressshot_seed_fk_edges.json path ClothingItem User
```

## Commands

| Command                      | Role                                                             | Source              |
| ---------------------------- | ---------------------------------------------------------------- | ------------------- |
| `tables`                     | lists tables, one `schema.table` per line                         | fixture, SQL, PG    |
| `schemas`                    | lists the application schemas of a database                       | PostgreSQL          |
| `databases` (alias `dbs`)    | lists the visible, connectable databases                          | PostgreSQL          |
| `tree`                       | prints databases → schemas → tables → outgoing FKs                | PostgreSQL          |
| `path <source> <target>`     | lists every FK path between two tables, simplest first            | fixture, SQL, PG    |
| `fixture`                    | converts the selected source into fixture JSON                    | fixture, SQL, PG    |
| `help [command]`             | prints the general help, or the help of one command               | —                   |
| `version`                    | prints the version                                                | —                   |

Every listing command prints an explicit marker when the result is empty:
`NO_TABLES`, `NO_SCHEMAS`, `NO_DATABASES`. Scripts can therefore tell "empty"
apart from a failure without parsing the output line by line.

## Global options

| Option                 | Role                                                                     |
| ---------------------- | ------------------------------------------------------------------------ |
| `--fixture <path>`     | read FK metadata from a JSON fixture                                      |
| `--sql <path>`         | read a PostgreSQL DDL and convert it to metadata                          |
| `--database-url <url>` | read metadata from this PostgreSQL connection URL                         |
| `--max-links <n>`      | maximum number of FK links per path (`path`), default `5`, bounded 1 to 8 |
| `--format <fmt>`       | output format (`path`): `text`, `equation`, `json`, `sql`, `mermaid`       |
| `-h`, `--help`         | print help (general, or for the command when one is given)                |
| `-V`, `--version`      | print the version                                                         |

Only one explicit source may be given at a time: combining `--fixture`, `--sql`
and `--database-url` is a usage error. Without an explicit source, the
`DATABASE_URL` environment variable is used.

`--format` only applies to `path`: the listing commands print one entry per line
so they stay composable with `grep`, `sort` or a shell loop.

## Help and version

```bash
schema-pathfinder --help          # general help
schema-pathfinder help            # same
schema-pathfinder help path       # help of the path command
schema-pathfinder tables --help   # same as "help tables", the command is not run
schema-pathfinder version         # or --version / -V
```

## List tables

```bash
schema-pathfinder tables --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder tables --sql fixtures/postgres/dressshot_schema.sql
schema-pathfinder tables --database-url postgres://readonly:change-me@localhost:5432/postgres
```

The output uses the `schema.table` format to avoid ambiguity, in a sorted and
therefore stable order from one call to the next. In PostgreSQL mode, tables are
searched across all visible application schemas, not only `public`.

## List schemas

```bash
schema-pathfinder schemas --database-url postgres://readonly:change-me@localhost:5432/postgres
```

This command requires a real PostgreSQL source: with `--fixture` or `--sql` it
returns `SOURCE_KIND_UNSUPPORTED`. The `information_schema` and `pg_*` schemas
are excluded.

## List databases

```bash
schema-pathfinder databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder dbs --database-url postgres://readonly:change-me@localhost:5432/postgres
```

PostgreSQL does not always grant access to every database: the result depends on
the privileges of the role used and on the server configuration. Template
databases and databases that disallow connections are excluded.

## Show the full tree

```bash
schema-pathfinder tree --database-url postgres://readonly:change-me@localhost:5432/postgres
```

Tree mode lists the visible databases, reconnects to each accessible database,
then prints the schemas, tables and outgoing relationships. A database that
cannot be introspected is still listed with its error
(`error DB_CONNECT_FAILED: ...`) instead of failing the whole command.

```text
postgres
|-- database app
|  `-- schema public
|     |-- table ClothingItem
|     |  `-- fk clothingSessionId -> public.ClothingSession.id (ClothingItem_clothingSessionId_fkey)
|     `-- table ClothingSession
`-- database postgres
```

## Find a path

```bash
schema-pathfinder path ClothingItem User --format equation --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder path public.ClothingItem public.User --format sql --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder path BON_DE_LIVRAISON COMMANDE_CLIENT --max-links 3 --database-url postgres://readonly:change-me@localhost:5432/postgres
```

The `path` command lists **all** foreign-key paths between the two tables, from
the simplest to the most complex: first the 1-link paths, then the 2-link ones,
and so on. For equal lengths, the highest-scoring path comes first, and the
remaining order is deterministic (constraint signature). Each path is numbered
(`Path 1`, `Path 2`, ...) and at most 50 paths are returned.

This is useful when the most "obvious" path goes through a table that is not yet
populated at data-entry time: you can then pick another path that is shorter or
that avoids that table.

- `--max-links <n>`: maximum number of links per path. Defaults to `5`, bounded
  between `1` and `8`. A value outside the range is clamped into it; a
  non-numeric value is a usage error. Beyond 5 links, joins usually become too
  complex to write by hand.

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

This command turns a PostgreSQL DDL into a JSON fixture, which can then be reused
without a live database.

## Compatibility

The legacy flags are still accepted and equivalent to the subcommands:

```bash
schema-pathfinder --tables    --fixture fixtures/postgres/dressshot_seed_fk_edges.json
schema-pathfinder --schemas   --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --databases --database-url postgres://readonly:change-me@localhost:5432/postgres
schema-pathfinder --tree      --database-url postgres://readonly:change-me@localhost:5432/postgres
```

They cannot be combined with each other, nor with a subcommand.

## Exit codes

| Code | Meaning                                                                     |
| ---- | --------------------------------------------------------------------------- |
| `0`  | success                                                                      |
| `1`  | runtime error (missing file, unreachable database, no path, ...)             |
| `2`  | usage error (bad argument, unknown command, unsupported format)              |

## Error codes

Errors are written to stderr as `CODE: message`, with a stable English code:

- `ARGS_INVALID`: invalid argument, command, or number of arguments (exit 2).
- `UNSUPPORTED_FORMAT`: unknown output format (exit 2).
- `DB_CONFIG_MISSING`: no source and no `DATABASE_URL`.
- `SOURCE_KIND_UNSUPPORTED`: PostgreSQL command called with a fixture or SQL source.
- `DB_CONNECT_FAILED`, `DB_METADATA_FAILED`: connection or introspection failure.
- `FIXTURE_READ_FAILED`, `FIXTURE_JSON_INVALID`, `FIXTURE_RENDER_FAILED`: unreadable fixture.
- `SQL_READ_FAILED`, `SQL_PARSE_FAILED`: unreadable or unparsable DDL.
- `TABLE_NOT_FOUND`: source or target table missing from the metadata.
- `NO_DECLARED_FK_PATH`: no declared FK path between the two tables.

These codes make integration into scripts or a UI easier.
