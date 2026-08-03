export const postgresForeignKeyMetadataSql = `
select
  source_ns.nspname as source_schema,
  source_table.relname as source_table,
  source_column.attname as source_column,
  target_ns.nspname as target_schema,
  target_table.relname as target_table,
  target_column.attname as target_column,
  constraint_info.conname as constraint_name
from pg_constraint constraint_info
join pg_class source_table on source_table.oid = constraint_info.conrelid
join pg_namespace source_ns on source_ns.oid = source_table.relnamespace
join pg_class target_table on target_table.oid = constraint_info.confrelid
join pg_namespace target_ns on target_ns.oid = target_table.relnamespace
join unnest(constraint_info.conkey) with ordinality source_cols(attnum, ord) on true
join unnest(constraint_info.confkey) with ordinality target_cols(attnum, ord) on source_cols.ord = target_cols.ord
join pg_attribute source_column on source_column.attrelid = source_table.oid and source_column.attnum = source_cols.attnum
join pg_attribute target_column on target_column.attrelid = target_table.oid and target_column.attnum = target_cols.attnum
where constraint_info.contype = 'f'
  and source_ns.nspname = any($1::text[])
order by source_ns.nspname, source_table.relname, constraint_info.conname, source_cols.ord;
`;

export function mapPostgresForeignKeyRow(row) {
  return {
    constraintName: row.constraint_name,
    from: {
      schema: row.source_schema,
      table: row.source_table
    },
    fromColumn: row.source_column,
    to: {
      schema: row.target_schema,
      table: row.target_table
    },
    toColumn: row.target_column,
    evidence: ["declared_fk"]
  };
}

export async function discoverPostgresForeignKeys(client, schemas = ["public"]) {
  const result = await client.query(postgresForeignKeyMetadataSql, [schemas]);

  return result.rows.map((row) => mapPostgresForeignKeyRow(row));
}
