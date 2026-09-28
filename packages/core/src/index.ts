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
export { findPaths, findPathsByComplexity } from "./path_search.mjs";
export { formatList, outputFormatSchema, outputFormats, parseOutputFormat } from "./output_formats.mjs";
export {
  clampMaxLinks,
  defaultMaxLinks,
  maxLinksCeiling,
  maxRankedPaths,
  rankPaths,
  rankPathsByComplexity,
  scorePath,
  scoringRules
} from "./scoring.mjs";
export {
  renderEquation,
  renderJson,
  renderMermaid,
  renderPath,
  renderPaths,
  renderSql,
  renderText
} from "./renderers.mjs";

export type OutputFormat = "text" | "equation" | "json" | "sql" | "mermaid";

export const pathfinderCorePackageName: string = "@schema-pathfinder/core";

export function getSupportedFormats(): OutputFormat[] {
  return ["text", "equation", "json", "sql", "mermaid"];
}
