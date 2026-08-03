export function createTableIdentifier(schema, table) {
  return { schema, table };
}

export function tableKey(identifier) {
  return `${identifier.schema}.${identifier.table}`;
}

export function createSchemaGraph(edges) {
  const nodes = new Map();
  const adjacency = new Map();

  for (const edge of edges) {
    const fromKey = tableKey(edge.from);
    const toKey = tableKey(edge.to);

    nodes.set(fromKey, edge.from);
    nodes.set(toKey, edge.to);

    if (!adjacency.has(fromKey)) {
      adjacency.set(fromKey, []);
    }

    if (!adjacency.has(toKey)) {
      adjacency.set(toKey, []);
    }

    adjacency.get(fromKey).push(edge);
  }

  return {
    nodes,
    adjacency,
    edges
  };
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
