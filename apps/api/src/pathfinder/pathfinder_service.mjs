import { readFileSync } from "node:fs";
import { parseOutputFormat } from "@schema-pathfinder/core/output_formats";
import { findPaths } from "@schema-pathfinder/core/path_search";
import { renderPath } from "@schema-pathfinder/core/renderers";

export function createPathfinderService(edgeLoader = defaultEdgeLoader) {
  return {
    async findPath(request) {
      const validation = validateRequest(request);

      if (validation.valid === false) {
        return {
          error: validation.error
        };
      }

      const edges = await edgeLoader();
      const result = findPaths({
        edges,
        sourceTable: request.sourceTable,
        targetTable: request.targetTable
      });

      if (result.paths.length === 0) {
        return result;
      }

      return {
        ...result,
        rendered: renderPath(result.paths[0], request.format)
      };
    }
  };
}

function validateRequest(request) {
  if (request.sourceTable === undefined || request.sourceTable.length === 0) {
    return validationError("SOURCE_TABLE_REQUIRED", "Source table is required");
  }

  if (request.targetTable === undefined || request.targetTable.length === 0) {
    return validationError("TARGET_TABLE_REQUIRED", "Target table is required");
  }

  const format = parseOutputFormat(request.format);

  if (format.success === false) {
    return validationError("UNSUPPORTED_FORMAT", "Unsupported output format");
  }

  return {
    valid: true
  };
}

function validationError(code, title) {
  return {
    valid: false,
    error: {
      code,
      title
    }
  };
}

async function defaultEdgeLoader() {
  const fixture = JSON.parse(readFileSync("fixtures/postgres/dressshot_seed_fk_edges.json", "utf8"));
  return fixture.edges;
}
