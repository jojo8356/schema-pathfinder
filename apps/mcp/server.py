#!/usr/bin/env python3
"""Schema Pathfinder MCP server — stdio JSON-RPC 2.0 (NDJSON).

Zero third-party dependencies. Wraps the existing `schema-pathfinder` CLI.
See docs/mcp.md for the specification.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

PROTOCOL_VERSION = "2024-11-05"
SERVER_NAME = "schema-pathfinder"
SERVER_VERSION = "0.1.4"
CLI_TIMEOUT_SEC = 30
FORMATS = ("text", "equation", "sql", "mermaid", "json")

ROOT = Path(os.environ.get("SCHEMA_PATHFINDER_ROOT") or Path(__file__).resolve().parents[2])
BIN = os.environ.get("SCHEMA_PATHFINDER_BIN", "schema-pathfinder")


def log(message: str) -> None:
    sys.stderr.write(message + "\n")
    sys.stderr.flush()


def read_cli_docs() -> str:
    path = ROOT / "docs" / "cli-reference.md"
    if path.is_file():
        return path.read_text(encoding="utf-8")
    return "CLI commands: tables, schemas, databases, tree, path, fixture, help."


def read_spec() -> str:
    path = ROOT / "docs" / "mcp.md"
    if path.is_file():
        return path.read_text(encoding="utf-8")
    return "Schema Pathfinder MCP server."


TOOLS: list[dict[str, Any]] = [
    {
        "name": "pathfinder_tables",
        "description": "List schema.table names from fixture, SQL DDL, or PostgreSQL metadata.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fixture": {"type": "string", "description": "JSON fixture path"},
                "sql": {"type": "string", "description": "PostgreSQL DDL file path"},
                "database_url": {"type": "string", "description": "postgres:// URL"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_schemas",
        "description": "List application schemas. Requires a live PostgreSQL source.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fixture": {"type": "string"},
                "sql": {"type": "string"},
                "database_url": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_databases",
        "description": "List visible, connectable PostgreSQL databases.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fixture": {"type": "string"},
                "sql": {"type": "string"},
                "database_url": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_tree",
        "description": "Print databases → schemas → tables → outgoing foreign keys.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fixture": {"type": "string"},
                "sql": {"type": "string"},
                "database_url": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_path",
        "description": "Rank FK join paths from from_table to to_table (simplest first).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "from_table": {"type": "string", "description": "Source table (name or schema.table)"},
                "to_table": {"type": "string", "description": "Target table"},
                "format": {
                    "type": "string",
                    "enum": list(FORMATS),
                    "description": "Output format, default equation",
                },
                "max_links": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 8,
                    "description": "Max FK hops, default 5",
                },
                "fixture": {"type": "string"},
                "sql": {"type": "string"},
                "database_url": {"type": "string"},
            },
            "required": ["from_table", "to_table"],
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_fixture",
        "description": "Export the chosen source as a reusable JSON fixture.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fixture": {"type": "string"},
                "sql": {"type": "string"},
                "database_url": {"type": "string"},
            },
            "additionalProperties": False,
        },
    },
    {
        "name": "pathfinder_help",
        "description": "Show CLI help, optionally for one command topic.",
        "inputSchema": {
            "type": "object",
            "properties": {"topic": {"type": "string"}},
            "additionalProperties": False,
        },
    },
]

RESOURCES = [
    {
        "uri": "pathfinder://docs/cli",
        "name": "CLI reference",
        "mimeType": "text/markdown",
    },
    {
        "uri": "pathfinder://spec",
        "name": "MCP specification",
        "mimeType": "text/markdown",
    },
]


def source_flags(args: dict[str, Any]) -> tuple[list[str] | None, str | None]:
    present = [key for key in ("fixture", "sql", "database_url") if args.get(key)]
    if len(present) > 1:
        return None, "USAGE: une seule source (fixture XOR sql XOR database_url)"
    if not present:
        return [], None
    key = present[0]
    flag = {"fixture": "--fixture", "sql": "--sql", "database_url": "--database-url"}[key]
    return [flag, str(args[key])], None


def run_cli(argv: list[str]) -> tuple[int, str, str]:
    try:
        completed = subprocess.run(
            [BIN, *argv],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=CLI_TIMEOUT_SEC,
            check=False,
        )
    except FileNotFoundError:
        return 127, "", f"BIN_NOT_FOUND: {BIN}"
    except subprocess.TimeoutExpired:
        return 124, "", "TIMEOUT: schema-pathfinder exceeded 30s"
    return completed.returncode, completed.stdout, completed.stderr


def tool_text(is_error: bool, text: str) -> dict[str, Any]:
    return {
        "content": [{"type": "text", "text": text}],
        "isError": is_error,
    }


def call_tool(name: str, args: dict[str, Any] | None) -> dict[str, Any]:
    args = args or {}
    if name == "pathfinder_help":
        argv = ["help"]
        topic = args.get("topic")
        if topic:
            argv.append(str(topic))
        code, out, err = run_cli(argv)
        text = out if code == 0 else (err or out or f"exit {code}")
        return tool_text(code != 0, text)

    flags, usage_error = source_flags(args)
    if usage_error:
        return tool_text(True, usage_error)

    if name == "pathfinder_tables":
        argv = ["tables", *flags]
    elif name == "pathfinder_schemas":
        argv = ["schemas", *flags]
    elif name == "pathfinder_databases":
        argv = ["databases", *flags]
    elif name == "pathfinder_tree":
        argv = ["tree", *flags]
    elif name == "pathfinder_fixture":
        argv = ["fixture", *flags]
    elif name == "pathfinder_path":
        from_table = args.get("from_table")
        to_table = args.get("to_table")
        if not from_table or not to_table:
            return tool_text(True, "USAGE: from_table and to_table are required")
        fmt = args.get("format") or "equation"
        if fmt not in FORMATS:
            return tool_text(True, f"USAGE: format must be one of {', '.join(FORMATS)}")
        max_links = args.get("max_links", 5)
        try:
            max_links = int(max_links)
        except (TypeError, ValueError):
            return tool_text(True, "USAGE: max_links must be an integer")
        if max_links < 1 or max_links > 8:
            return tool_text(True, "USAGE: max_links must be between 1 and 8")
        argv = [
            "path",
            str(from_table),
            str(to_table),
            "--format",
            fmt,
            "--max-links",
            str(max_links),
            *flags,
        ]
    else:
        return tool_text(True, f"UNKNOWN_TOOL: {name}")

    code, out, err = run_cli(argv)
    if code == 0:
        return tool_text(False, out)
    return tool_text(True, (err or out or f"exit {code}").strip())


def handle(message: dict[str, Any]) -> dict[str, Any] | None:
    method = message.get("method")
    msg_id = message.get("id")
    params = message.get("params") or {}

    if method and str(method).startswith("notifications/"):
        return None

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {"listChanged": False},
                    "resources": {"subscribe": False, "listChanged": False},
                    "prompts": {"listChanged": False},
                },
                "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
            },
        }

    if method == "ping":
        return {"jsonrpc": "2.0", "id": msg_id, "result": {}}

    if method == "tools/list":
        return {"jsonrpc": "2.0", "id": msg_id, "result": {"tools": TOOLS}}

    if method == "tools/call":
        name = params.get("name", "")
        arguments = params.get("arguments") or {}
        return {"jsonrpc": "2.0", "id": msg_id, "result": call_tool(name, arguments)}

    if method == "resources/list":
        return {"jsonrpc": "2.0", "id": msg_id, "result": {"resources": RESOURCES}}

    if method == "resources/read":
        uri = params.get("uri", "")
        if uri == "pathfinder://docs/cli":
            text = read_cli_docs()
        elif uri == "pathfinder://spec":
            text = read_spec()
        else:
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "error": {"code": -32602, "message": f"Unknown resource: {uri}"},
            }
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "contents": [{"uri": uri, "mimeType": "text/markdown", "text": text}],
            },
        }

    if method == "prompts/list":
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "prompts": [
                    {
                        "name": "find_join_path",
                        "description": "Ask the model to find and explain FK join paths.",
                        "arguments": [
                            {"name": "from_table", "required": True},
                            {"name": "to_table", "required": True},
                            {"name": "format", "required": False},
                        ],
                    }
                ]
            },
        }

    if method == "prompts/get":
        name = params.get("name")
        arguments = params.get("arguments") or {}
        if name != "find_join_path":
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "error": {"code": -32602, "message": f"Unknown prompt: {name}"},
            }
        from_table = arguments.get("from_table", "A")
        to_table = arguments.get("to_table", "B")
        fmt = arguments.get("format", "equation")
        text = (
            f"Use pathfinder_tables then pathfinder_path to join {from_table} to {to_table} "
            f"with format={fmt}. Explain the shortest path and mention alternatives."
        )
        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "description": "Find a join path between two tables.",
                "messages": [{"role": "user", "content": {"type": "text", "text": text}}],
            },
        }

    if msg_id is None:
        return None
    return {
        "jsonrpc": "2.0",
        "id": msg_id,
        "error": {"code": -32601, "message": f"Method not found: {method}"},
    }


def read_message(stdin) -> dict[str, Any] | None:
    first = stdin.readline()
    if first == "":
        return None
    if first.lower().startswith("content-length:"):
        length = int(first.split(":", 1)[1].strip())
        while True:
            line = stdin.readline()
            if line in ("", "\n", "\r\n"):
                break
        body = stdin.read(length)
        return json.loads(body)
    line = first.strip()
    if not line:
        return read_message(stdin)
    return json.loads(line)


def write_message(payload: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def main() -> int:
    stdin = sys.stdin
    while True:
        try:
            message = read_message(stdin)
        except json.JSONDecodeError as error:
            write_message(
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32700, "message": f"Parse error: {error}"},
                }
            )
            continue
        if message is None:
            return 0
        reply = handle(message)
        if reply is not None:
            write_message(reply)


if __name__ == "__main__":
    raise SystemExit(main())
