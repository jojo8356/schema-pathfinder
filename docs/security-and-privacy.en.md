# Security and Privacy

## Security model

Schema Pathfinder is designed to inspect structural metadata: visible databases, schemas, tables, and foreign keys. It does not need to read the business rows of the tables to fulfill its main purpose.

## PostgreSQL best practices

Create a dedicated role with minimal privileges:

- connection to the database;
- read access to the necessary system catalogs;
- no write privilege;
- no broad access to business data unless the environment requires it.

Illustrative example:

```sql
CREATE ROLE schema_pathfinder_reader LOGIN PASSWORD 'change_me';
GRANT CONNECT ON DATABASE your_database TO schema_pathfinder_reader;
GRANT USAGE ON SCHEMA public TO schema_pathfinder_reader;
```

Adapt the privileges to your server. Do not reuse this example as-is in production.

## Secrets

- Never commit a PostgreSQL URL that contains a real password.
- Use `DATABASE_URL` as an environment variable for local tests.
- Use the platform's runtime secrets for a web deployment.
- Remove logs containing a full URL before sharing them publicly.

## Web and API

An exposed web instance can reveal schema metadata. If it is publicly accessible, enable a token or place it behind authentication.

## Generated SQL data

The `sql` format produces a read-only query skeleton. The tool does not execute it automatically. The user remains responsible for any execution in their own environment.
