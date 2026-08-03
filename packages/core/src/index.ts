export { createSchemaGraph, createTableIdentifier, resolveTable, tableKey } from "./graph.mjs";
export {
  discoverPostgresForeignKeys,
  mapPostgresForeignKeyRow,
  postgresForeignKeyMetadataSql
} from "./postgres_fk_metadata.mjs";
export { findPaths } from "./path_search.mjs";

export type OutputFormat = "text" | "equation" | "json" | "sql" | "mermaid";

export const pathfinderCorePackageName: string = "@schema-pathfinder/core";

export function getSupportedFormats(): OutputFormat[] {
  return ["text", "equation", "json", "sql", "mermaid"];
}
