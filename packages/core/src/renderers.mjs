export function renderText(path, index = 0) {
  const tablePath = [path.source.table];

  for (const edge of path.edges) {
    tablePath.push(edge.to.table);
  }

  const lines = [
    `Path ${index + 1} score ${path.score} ${path.evidence.join(",")} length ${path.length}`,
    tablePath.join(" -> "),
    "",
    "Edges:"
  ];

  path.edges.forEach((edge, edgeIndex) => {
    lines.push(
      `${edgeIndex + 1}. ${edge.from.table}.${edge.fromColumn} -> ${edge.to.table}.${edge.toColumn}`
    );
  });

  return lines.join("\n");
}

export function renderEquation(path) {
  return path.edges
    .map((edge) => `${edge.from.table}.${edge.fromColumn} = ${edge.to.table}.${edge.toColumn}`)
    .join(" -> ");
}

export function renderJson(result) {
  return JSON.stringify(result, null, 2);
}

export function renderSql(path, limit = 50) {
  if (path.edges.length === 0) {
    return "";
  }

  const lines = [
    "select *",
    `from ${quoteIdentifier(path.source.table)} t0`
  ];

  path.edges.forEach((edge, edgeIndex) => {
    const fromAlias = `t${edgeIndex}`;
    const toAlias = `t${edgeIndex + 1}`;

    lines.push(
      `join ${quoteIdentifier(edge.to.table)} ${toAlias} on ${fromAlias}.${quoteIdentifier(edge.fromColumn)} = ${toAlias}.${quoteIdentifier(edge.toColumn)}`
    );
  });

  lines.push(`limit ${limit};`);

  return lines.join("\n");
}

export function renderMermaid(path) {
  const lines = ["flowchart LR"];

  for (const edge of path.edges) {
    lines.push(`  ${edge.from.table} --> ${edge.to.table}`);
  }

  return lines.join("\n");
}

export function renderPath(path, format) {
  if (format === "text") {
    return renderText(path);
  }

  if (format === "equation") {
    return renderEquation(path);
  }

  if (format === "sql") {
    return renderSql(path);
  }

  if (format === "mermaid") {
    return renderMermaid(path);
  }

  if (format === "json") {
    return JSON.stringify(path, null, 2);
  }

  throw new Error(`Unsupported output format: ${format}`);
}

function quoteIdentifier(identifier) {
  return `"${identifier.replaceAll("\"", "\"\"")}"`;
}
