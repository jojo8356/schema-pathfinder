import { Graph } from "@dagrejs/graphlib";

export function createTableIdentifier(schema, table) {
  return { schema, table };
}

export function tableKey(identifier) {
  return `${identifier.schema}.${identifier.table}`;
}

export function createSchemaGraph(edges) {
  const nodes = new Map();
  const graph = new Graph({ directed: true, multigraph: true });

  for (const edge of edges) {
    const fromKey = tableKey(edge.from);
    const toKey = tableKey(edge.to);

    nodes.set(fromKey, edge.from);
    nodes.set(toKey, edge.to);
    graph.setNode(fromKey, edge.from);
    graph.setNode(toKey, edge.to);
    graph.setEdge(fromKey, toKey, edge, edge.constraintName);
  }

  return {
    graph,
    nodes,
    edges
  };
}

export function outgoingEdges(schemaGraph, identifier) {
  const rawEdges = schemaGraph.graph.outEdges(tableKey(identifier));

  if (rawEdges === undefined) {
    return [];
  }

  return rawEdges.map((rawEdge) => schemaGraph.graph.edge(rawEdge));
}

export function resolveTable(graph, tableName, schema = "public") {
  const directKey = `${schema}.${tableName}`;

  if (graph.nodes.has(directKey)) {
    return graph.nodes.get(directKey);
  }

  for (const node of graph.nodes.values()) {
    if (node.table === tableName) {
      return node;
    }
  }

  return undefined;
}
