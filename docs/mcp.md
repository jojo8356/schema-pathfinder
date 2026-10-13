# Cahier des charges — serveur MCP Schema Pathfinder

## 1. Contexte

**Model Context Protocol (MCP)** est le protocole JSON-RPC 2.0 par lequel une
IA (hôte : Claude Desktop, Cursor, VS Code, Arena, etc.) découvre et appelle
des *tools*, lit des *resources* et instancie des *prompts* exposés par un
programme local ou distant.

Schema Pathfinder répond déjà à « comment joindre la table A à la table B ? »
via un CLI Rust, une UI desktop, une UI web et une API HTTP. Il manque une
surface **agent** : une IA doit pouvoir lister les tables et calculer un chemin
FK **sans réimplémenter le graphe**, en réutilisant le binaire CLI existant.

Sources de référence :

- Spécification MCP (couche données JSON-RPC, primitive tools / resources / prompts)
- Transport **stdio** (processus local, une ligne JSON par message)
- Contrat CLI documenté dans `docs/cli-reference.md`

## 2. Objectifs

1. Exposer **toutes les commandes CLI stables** (`tables`, `schemas`,
   `databases`, `tree`, `path`, `fixture`, `help`) comme tools MCP.
2. Ne **jamais** lire les lignes métier : uniquement les métadonnées, comme le CLI.
3. Une seule source à la fois (`fixture` XOR `sql` XOR `database_url`), identique
   au CLI.
4. Être testable par un client MCP réel (initialize → list → call) sans SDK.
5. Zéro dépendance Python hors bibliothèque standard, pour rester installable
   partout où `python3` ≥ 3.11 existe.

Hors périmètre : transport HTTP Streamable, OAuth, écriture en base, GUI.

## 3. Acteurs

| Acteur | Rôle |
| ------ | ---- |
| Hôte MCP | Claude, Cursor, VS Code, script de test |
| Serveur `schema-pathfinder-mcp` | Processus stdio, ce document |
| CLI `schema-pathfinder` | Moteur ; le MCP ne recalcule pas les chemins |

## 4. Architecture

```
Hôte MCP  --stdio JSON-RPC-->  apps/mcp/server.py  --subprocess-->  schema-pathfinder
```

Variable d'environnement `SCHEMA_PATHFINDER_BIN` : chemin du binaire (défaut
`schema-pathfinder` sur le `PATH`). Répertoire de travail : racine du dépôt si
`SCHEMA_PATHFINDER_ROOT` est défini, sinon le cwd du processus.

## 5. Transport et protocole

- JSON-RPC 2.0, messages **délimités par un saut de ligne** (NDJSON).
- Lecture : NDJSON **ou** framing `Content-Length` (compatibilité LSP).
- `protocolVersion` annoncé : `2024-11-05`.
- Méthodes obligatoires : `initialize`, `notifications/initialized`,
  `ping`, `tools/list`, `tools/call`, `resources/list`, `resources/read`,
  `prompts/list`, `prompts/get`.
- Méthode inconnue → erreur JSON-RPC `-32601`.
- Un tool CLI qui échoue (code ≠ 0) → `tools/call` avec `isError: true`, pas une
  exception JSON-RPC (l'IA doit lire le message `CODE: ...`).

## 6. Tools

Tous les tools acceptent les arguments de source optionnels :

| Argument | Type | Règle |
| -------- | ---- | ----- |
| `fixture` | string | chemin fichier JSON |
| `sql` | string | chemin DDL |
| `database_url` | string | URL postgres |
| `schema` | string | ignoré par le CLI actuel ; réservé, non envoyé |

Au plus un des trois `fixture` / `sql` / `database_url`. Sinon le tool renvoie
`isError` avec `USAGE: une seule source`.

| Tool | Commande CLI | Arguments spécifiques | Sortie |
| ---- | ------------ | --------------------- | ------ |
| `pathfinder_tables` | `tables` | — | texte `schema.table` |
| `pathfinder_schemas` | `schemas` | — | texte, ou `SOURCE_KIND_UNSUPPORTED` |
| `pathfinder_databases` | `databases` | — | texte |
| `pathfinder_tree` | `tree` | — | arbre texte |
| `pathfinder_path` | `path <from> <to>` | `from_table`, `to_table` (requis), `format` (`text`\|`equation`\|`sql`\|`mermaid`\|`json`, défaut `equation`), `max_links` (1–8, défaut 5) | chemins classés |
| `pathfinder_fixture` | `fixture` | — | JSON fixture |
| `pathfinder_help` | `help [topic]` | `topic` optionnel | aide |

Noms préfixés `pathfinder_` pour éviter les collisions dans un hôte multi-serveurs.

## 7. Resources

| URI | Contenu |
| --- | ------- |
| `pathfinder://docs/cli` | texte de `docs/cli-reference.md` si le fichier existe, sinon un résumé court |
| `pathfinder://spec` | ce cahier des charges (fichier `docs/mcp.md`) |

Lecture seule. URI inconnu → erreur `-32602`.

## 8. Prompts

| Nom | Arguments | Message utilisateur produit |
| --- | --------- | --------------------------- |
| `find_join_path` | `from_table`, `to_table`, `format?` | Consigne d'appeler `pathfinder_path` puis d'expliquer le meilleur chemin |

## 9. Sécurité

- Pas de shell (`subprocess` liste d'arguments).
- Pas d'interpolation de l'URL dans un shell.
- Time-out 30 s par appel CLI.
- Le MCP n'ouvre pas de socket réseau lui-même.
- Identique au CLI : métadonnées uniquement.

## 10. Intégration hôte (exemple Claude Desktop)

```json
{
  "mcpServers": {
    "schema-pathfinder": {
      "command": "python3",
      "args": ["/chemin/schema-pathfinder/apps/mcp/server.py"],
      "env": {
        "SCHEMA_PATHFINDER_BIN": "/chemin/schema-pathfinder/target/debug/schema-pathfinder",
        "SCHEMA_PATHFINDER_ROOT": "/chemin/schema-pathfinder"
      }
    }
  }
}
```

## 11. Recette d'acceptation

1. `initialize` → `serverInfo.name == "schema-pathfinder"`.
2. `tools/list` → 7 tools, chacun avec un `inputSchema` JSON Schema.
3. `pathfinder_path` sans `from_table` → `isError`.
4. `pathfinder_path` avec fixture mockée → stdout du CLI, `isError` false.
5. Deux sources à la fois → `isError` usage.
6. `resources/read` `pathfinder://spec` → document non vide.
7. `prompts/get` `find_join_path` → au moins un message `role=user`.
8. Accolades / JSON-RPC valides ; le serveur ignore `notifications/*` sans `id`.

## 12. Tests automatisés

`tests/mcp_server.test.mjs` : client Node qui spawn `python3 apps/mcp/server.py`
avec un binaire CLI factice (`SCHEMA_PATHFINDER_BIN`).
