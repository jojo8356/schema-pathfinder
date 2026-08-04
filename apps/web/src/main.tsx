import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { Database, GitBranch, Loader2, Search } from "lucide-react";
import "./styles.css";

type TableIdentifier = {
  schema: string;
  table: string;
};

type TablesResponse = {
  tables: TableIdentifier[];
};

type PathResponse = {
  paths: unknown[];
  rendered?: string;
  noPathReason?: string;
};

type SourceKind = "fixture" | "sql" | "postgres";
type OutputFormat = "equation" | "sql" | "text" | "mermaid" | "json";

const sourceLabels: Record<SourceKind, string> = {
  fixture: "Fixture file",
  sql: "SQL file",
  postgres: "Postgres URL"
};

const defaultSources: Record<SourceKind, string> = {
  fixture: "fixtures/postgres/dressshot_seed_fk_edges.json",
  sql: "",
  postgres: ""
};

function App() {
  const [sourceKind, setSourceKind] = useState<SourceKind>("fixture");
  const [sources, setSources] = useState<Record<SourceKind, string>>(defaultSources);
  const [tables, setTables] = useState<TableIdentifier[]>([]);
  const [schema, setSchema] = useState("public");
  const [sourceTable, setSourceTable] = useState("");
  const [targetTable, setTargetTable] = useState("");
  const [format, setFormat] = useState<OutputFormat>("equation");
  const [output, setOutput] = useState("");
  const [status, setStatus] = useState("Ready");
  const [loading, setLoading] = useState(false);

  const schemas = useMemo(() => {
    const values = Array.from(new Set(tables.map((table) => table.schema))).sort();
    if (values.length === 0) {
      return ["public"];
    }
    return values;
  }, [tables]);

  const tableOptions = useMemo(() => {
    return tables
      .filter((table) => table.schema === schema)
      .map((table) => table.table)
      .sort();
  }, [schema, tables]);

  useEffect(() => {
    void loadTables();
  }, []);

  useEffect(() => {
    if (schemas.includes(schema) === false) {
      setSchema(schemas[0]);
    }
  }, [schema, schemas]);

  useEffect(() => {
    if (tableOptions.length === 0) {
      setSourceTable("");
      setTargetTable("");
      return;
    }

    if (tableOptions.includes(sourceTable) === false) {
      setSourceTable(tableOptions[0]);
    }

    if (tableOptions.includes(targetTable) === false) {
      setTargetTable(tableOptions[tableOptions.length - 1]);
    }
  }, [sourceTable, tableOptions, targetTable]);

  function updateSourceValue(value: string) {
    setSources((current) => ({ ...current, [sourceKind]: value }));
  }

  async function loadTables() {
    setLoading(true);
    setStatus("Loading schema metadata");

    try {
      const response = await fetch("/admin/pathfinder/tables");
      const json = await readJson<TablesResponse>(response);
      setTables(json.tables);
      setStatus("Loaded schema metadata");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLoading(false);
    }
  }

  async function findPath() {
    setLoading(true);
    setStatus("Finding path");

    try {
      const response = await fetch("/admin/pathfinder/path", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          sourceTable: qualifiedTable(schema, sourceTable),
          targetTable: qualifiedTable(schema, targetTable),
          format
        })
      });
      const json = await readJson<PathResponse>(response);
      if (json.rendered !== undefined) {
        setOutput(json.rendered);
        setStatus("Path found");
        return;
      }
      setOutput(json.noPathReason ?? "NO_DECLARED_FK_PATH");
      setStatus("No declared FK path found");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <h1>Schema Pathfinder</h1>
      </header>

      <section className="workspace">
        <aside className="panel source-panel">
          <h2>Source</h2>
          <label className="field">
            <span>{sourceLabels[sourceKind]}</span>
            <input value={sources[sourceKind]} onChange={(event) => updateSourceValue(event.target.value)} />
          </label>
          <div className="source-actions">
            <select value={sourceKind} onChange={(event) => setSourceKind(event.target.value as SourceKind)}>
              <option value="fixture">fixture</option>
              <option value="sql">sql</option>
              <option value="postgres">postgres</option>
            </select>
            <button type="button" onClick={loadTables}>
              {loading ? <Loader2 size={16} className="spin" /> : <Database size={16} />}
              Load
            </button>
          </div>
          <div className="divider" />
          <div className="panel-title-row">
            <h2>Tables</h2>
            <span>schema.table</span>
          </div>
          <pre className="tables-list">{tables.map((table) => table.schema + "." + table.table).join("\\n")}</pre>
        </aside>

        <section className="main-column">
          <section className="panel path-panel">
            <h2>Path</h2>
            <div className="path-grid">
              <label className="field">
                <span>Schema</span>
                <select value={schema} onChange={(event) => setSchema(event.target.value)}>
                  {schemas.map((value) => <option key={value} value={value}>{value}</option>)}
                </select>
              </label>
              <label className="field">
                <span>From table</span>
                <select value={sourceTable} onChange={(event) => setSourceTable(event.target.value)}>
                  {tableOptions.map((value) => <option key={value} value={value}>{value}</option>)}
                </select>
              </label>
              <label className="field">
                <span>To table</span>
                <select value={targetTable} onChange={(event) => setTargetTable(event.target.value)}>
                  {tableOptions.map((value) => <option key={value} value={value}>{value}</option>)}
                </select>
              </label>
            </div>
            <div className="format-row">
              <label className="field inline-field">
                <span>Format</span>
                <select value={format} onChange={(event) => setFormat(event.target.value as OutputFormat)}>
                  <option value="equation">equation</option>
                  <option value="sql">sql</option>
                  <option value="text">text</option>
                  <option value="mermaid">mermaid</option>
                  <option value="json">json</option>
                </select>
              </label>
              <button type="button" onClick={findPath}>
                <Search size={16} />
                Find path
              </button>
            </div>
          </section>

          <section className="panel result-panel">
            <div className="panel-title-row">
              <h2>Result</h2>
              <span>{format}</span>
            </div>
            <pre className="result-box">{output}</pre>
          </section>
        </section>
      </section>
      <div className="status-line"><GitBranch size={14} />{status}</div>
    </main>
  );
}

async function readJson<T>(response: Response): Promise<T> {
  const text = await response.text();
  const json = JSON.parse(text);
  if (response.ok === false) {
    throw new Error(json.error?.title ?? response.statusText);
  }
  return json as T;
}

function qualifiedTable(schema: string, table: string): string {
  if (table.includes(".")) {
    return table;
  }
  return schema + "." + table;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected error";
}

createRoot(document.getElementById("root") as HTMLElement).render(<App />);
