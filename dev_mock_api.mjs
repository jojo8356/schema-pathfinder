// TEMPORARY demo backend for the live preview only. NOT part of the app.
// It reuses the JS core (the mirror of the Rust engine) so you can see the new
// "multiple paths by complexity" + "Max links" + scrollable table list without
// needing the Rust API compiled. Delete this file after the demo.
import http from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join } from "node:path";
import { findPathsByComplexity } from "@schema-pathfinder/core/path_search";
import { renderPaths, renderTables } from "@schema-pathfinder/core/renderers";

const DIST = join(process.cwd(), "apps/web/dist");
const PORT = Number(process.env.PORT || 8080);

const table = (name) => ({ schema: "public", table: name });
const edge = (constraintName, from, fromColumn, to, toColumn) => ({
  constraintName,
  from: table(from),
  fromColumn,
  to: table(to),
  toColumn,
  evidence: ["declared_fk"]
});

// A small OpenConcerto-flavoured schema with THREE routes from the delivery note
// (BON_DE_LIVRAISON) to the customer order (COMMANDE_CLIENT):
//   1 link  : direct FK
//   2 links : via DEVIS
//   3 links : via SAISIE_VENTE_FACTURE -> FACTURATION_COMMANDE_CLIENT (often empty at BL time)
const edges = [
  edge("BL_cmd_fkey", "BON_DE_LIVRAISON", "ID_COMMANDE_CLIENT", "COMMANDE_CLIENT", "ID"),
  edge("BL_devis_fkey", "BON_DE_LIVRAISON", "ID_DEVIS", "DEVIS", "ID"),
  edge("DEVIS_cmd_fkey", "DEVIS", "ID_COMMANDE_CLIENT", "COMMANDE_CLIENT", "ID"),
  edge("BL_svf_fkey", "BON_DE_LIVRAISON", "ID_SAISIE_VENTE_FACTURE", "SAISIE_VENTE_FACTURE", "ID"),
  edge("SVF_fcc_fkey", "SAISIE_VENTE_FACTURE", "ID_FACTURATION_COMMANDE_CLIENT", "FACTURATION_COMMANDE_CLIENT", "ID"),
  edge("FCC_cmd_fkey", "FACTURATION_COMMANDE_CLIENT", "ID_COMMANDE_CLIENT", "COMMANDE_CLIENT", "ID"),
  edge("CMD_client_fkey", "COMMANDE_CLIENT", "ID_CLIENT", "CLIENT", "ID"),
  edge("BL_element_fkey", "BON_DE_LIVRAISON_ELEMENT", "ID_BON_DE_LIVRAISON", "BON_DE_LIVRAISON", "ID"),
  edge("CMD_element_fkey", "COMMANDE_CLIENT_ELEMENT", "ID_COMMANDE_CLIENT", "COMMANDE_CLIENT", "ID"),
  edge("CLIENT_adr_fkey", "CLIENT", "ID_ADRESSE", "ADRESSE", "ID")
];

// Extra standalone tables so the "Tables" list is long enough to scroll.
const extraTableNames = [
  "ARTICLE", "ARTICLE_TARIF", "ARTICLE_FOURNISSEUR", "AVOIR_CLIENT", "AVOIR_FOURNISSEUR",
  "BANQUE", "BON_RECEPTION", "COMPTE_PCE", "CONTACT", "DEPARTEMENT", "DEVISE", "ECHEANCE_CLIENT",
  "ETAT_DEVIS", "FACTURE_FOURNISSEUR", "FAMILLE_ARTICLE", "FOURNISSEUR", "JOURNAL", "LANGUE",
  "MODE_REGLEMENT", "MOUVEMENT_STOCK", "PAYS", "REGLEMENT_CLIENT", "SAISIE_ACHAT", "SAISIE_KM",
  "SITE", "STOCK", "TARIF", "TAXE", "TVA", "UNITE_VENTE", "UTILISATEUR", "VILLE"
].map(table);

const tableMap = new Map();
for (const e of edges) {
  tableMap.set(`${e.from.schema}.${e.from.table}`, e.from);
  tableMap.set(`${e.to.schema}.${e.to.table}`, e.to);
}
for (const t of extraTableNames) {
  tableMap.set(`${t.schema}.${t.table}`, t);
}
const tables = Array.from(tableMap.values()).sort((a, b) =>
  `${a.schema}.${a.table}`.localeCompare(`${b.schema}.${b.table}`)
);

function splitQualified(name) {
  const dot = name.indexOf(".");
  if (dot === -1) {
    return { schema: "public", table: name };
  }
  return { schema: name.slice(0, dot), table: name.slice(dot + 1) };
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) {
    chunks.push(chunk);
  }
  if (chunks.length === 0) {
    return {};
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

function sendJson(res, status, value) {
  const body = JSON.stringify(value);
  res.writeHead(status, { "content-type": "application/json" });
  res.end(body);
}

const contentTypes = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".json": "application/json"
};

async function serveStatic(req, res) {
  const url = req.url.split("?")[0];
  const relative = url === "/" ? "index.html" : url.replace(/^\/+/, "");
  const filePath = join(DIST, relative);
  try {
    const bytes = await readFile(filePath);
    res.writeHead(200, { "content-type": contentTypes[extname(filePath)] || "application/octet-stream" });
    res.end(bytes);
  } catch {
    const index = await readFile(join(DIST, "index.html"));
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(index);
  }
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === "POST" && req.url === "/api/tables") {
      await readBody(req);
      return sendJson(res, 200, { tables });
    }
    if (req.method === "POST" && req.url === "/api/databases") {
      await readBody(req);
      return sendJson(res, 200, { databases: [] });
    }
    if (req.method === "POST" && req.url === "/api/path") {
      const body = await readBody(req);
      const source = splitQualified(body.sourceTable);
      const target = splitQualified(body.targetTable);
      const result = findPathsByComplexity({
        edges,
        sourceTable: source.table,
        sourceSchema: source.schema,
        targetTable: target.table,
        targetSchema: target.schema,
        maxLinks: body.maxLinks
      });
      if (result.paths.length === 0) {
        return sendJson(res, 200, { paths: [], noPathReason: result.noPathReason || "NO_DECLARED_FK_PATH" });
      }
      const rendered = renderPaths(result.paths, body.format || "text");
      return sendJson(res, 200, { paths: result.paths, rendered });
    }
    if (req.method === "GET" && req.url === "/api/tables") {
      return sendJson(res, 200, { tables });
    }
    return serveStatic(req, res);
  } catch (error) {
    sendJson(res, 500, { error: { code: "MOCK_ERROR", title: String(error && error.message) } });
  }
});

server.listen(PORT, "0.0.0.0", () => {
  console.log(`demo backend + web on http://0.0.0.0:${PORT}`);
  console.log(`renderTables sample:\n${renderTables(tables.slice(0, 3))}`);
});
