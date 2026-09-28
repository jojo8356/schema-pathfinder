# Contract Fixtures

Contract fixtures keep all surfaces aligned.

## Files

- `fixtures/postgres/dressshot_seed_fk_edges.json`: declared FK metadata seed.
- `fixtures/postgres/dressshot_expected_clothing_item_user.json`: expected connected path output.
- `fixtures/postgres/dressshot_expected_no_path.json`: expected no-path output.

## Rules

- Fixtures contain schema metadata only.
- Fixtures must not include application rows or sample business data.
- Core tests own expected output semantics.
- Surface tests must consume contract fixtures or core renderer output instead of rebuilding text, SQL, equation, JSON, or Mermaid strings locally.

## Validation

```bash
node --test tests/*.test.mjs
node scripts/check_contract_fixtures.mjs
```
