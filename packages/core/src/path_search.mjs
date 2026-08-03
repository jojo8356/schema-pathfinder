import { createSchemaGraph, resolveTable, tableKey } from "./graph.mjs";
import { rankPaths, scorePath } from "./scoring.mjs";

export function findPaths(input) {
  const graph = createSchemaGraph(input.edges);
  const source = resolveTable(graph, input.sourceTable, input.sourceSchema);
  const target = resolveTable(graph, input.targetTable, input.targetSchema);
  let maxDepth = 6;

  if (input.maxDepth !== undefined) {
    maxDepth = input.maxDepth;
  }

  if (source === undefined || target === undefined) {
    let resultSource = source;
    let resultTarget = target;
    let sourceSchema = "public";
    let targetSchema = "public";

    if (input.sourceSchema !== undefined) {
      sourceSchema = input.sourceSchema;
    }

    if (input.targetSchema !== undefined) {
      targetSchema = input.targetSchema;
    }

    if (resultSource === undefined) {
      resultSource = { schema: sourceSchema, table: input.sourceTable };
    }

    if (resultTarget === undefined) {
      resultTarget = { schema: targetSchema, table: input.targetTable };
    }

    return {
      source: resultSource,
      target: resultTarget,
      paths: [],
      noPathReason: "TABLE_NOT_FOUND"
    };
  }

  const targetKey = tableKey(target);
  const queue = [{ current: source, edges: [] }];
  const results = [];

  while (queue.length > 0) {
    const item = queue.shift();
    const currentKey = tableKey(item.current);

    if (currentKey === targetKey) {
      results.push(scorePath(source, target, item.edges));
      continue;
    }

    if (item.edges.length >= maxDepth) {
      continue;
    }

    let nextEdges = graph.adjacency.get(currentKey);

    if (nextEdges === undefined) {
      nextEdges = [];
    }

    for (const edge of nextEdges) {
      const alreadyVisited = item.edges.some((visitedEdge) => {
        return tableKey(visitedEdge.from) === tableKey(edge.to);
      });

      if (alreadyVisited) {
        continue;
      }

      queue.push({
        current: edge.to,
        edges: [...item.edges, edge]
      });
    }
  }

  if (results.length === 0) {
    return {
      source,
      target,
      paths: [],
      noPathReason: "NO_DECLARED_FK_PATH"
    };
  }

  return {
    source,
    target,
    paths: rankPaths(results)
  };
}
