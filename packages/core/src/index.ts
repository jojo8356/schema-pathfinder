export {
  createSchemaGraph,
  createTableIdentifier,
  listTablesFromEdges,
  outgoingEdges,
  resolveTable,
  tableKey
} from "./graph.mjs";
export {
  discoverPostgresForeignKeys,
  mapPostgresForeignKeyRow,
  postgresForeignKeyMetadataSql
} from "./postgres_fk_metadata.mjs";
export { findPaths } from "./path_search.mjs";
export { formatList, outputFormatSchema, outputFormats, parseOutputFormat } from "./output_formats.mjs";
export { rankPaths, scorePath, scoringRules } from "./scoring.mjs";
export {
  renderEquation,
  renderJson,
  renderMermaid,
  renderPath,
  renderSql,
  renderText
} from "./renderers.mjs";

export type OutputFormat = "text" | "equation" | "json" | "sql" | "mermaid";

export const pathfinderCorePackageName: string = "@schema-pathfinder/core";

export function getSupportedFormats(): OutputFormat[] {
  return ["text", "equation", "json", "sql", "mermaid"];
}
