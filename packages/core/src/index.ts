export { createSchemaGraph, createTableIdentifier, resolveTable, tableKey } from "./graph.mjs";

export type OutputFormat = "text" | "equation" | "json" | "sql" | "mermaid";

export const pathfinderCorePackageName: string = "@schema-pathfinder/core";

export function getSupportedFormats(): OutputFormat[] {
  return ["text", "equation", "json", "sql", "mermaid"];
}
