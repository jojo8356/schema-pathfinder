import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { chmodSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, it } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const serverPath = path.join(root, "apps/mcp/server.py");

function fakeCli(script) {
  const dir = mkdtempSync(path.join(tmpdir(), "spf-mcp-"));
  const bin = path.join(dir, "schema-pathfinder");
  writeFileSync(bin, `#!/bin/sh\n${script}\n`);
  chmodSync(bin, 0o755);
  return bin;
}

function rpc(proc, message) {
  return new Promise((resolve, reject) => {
    let buffer = "";
    const onData = (chunk) => {
      buffer += chunk.toString("utf8");
      const nl = buffer.indexOf("\n");
      if (nl === -1) {
        return;
      }
      proc.stdout.off("data", onData);
      try {
        resolve(JSON.parse(buffer.slice(0, nl)));
      } catch (error) {
        reject(error);
      }
    };
    proc.stdout.on("data", onData);
    proc.stdin.write(`${JSON.stringify(message)}\n`);
    setTimeout(() => reject(new Error(`timeout waiting for ${message.method}`)), 8000);
  });
}

function startServer(bin) {
  return spawn("python3", [serverPath], {
    cwd: root,
    env: {
      ...process.env,
      SCHEMA_PATHFINDER_BIN: bin,
      SCHEMA_PATHFINDER_ROOT: root,
    },
    stdio: ["pipe", "pipe", "pipe"],
  });
}

describe("schema-pathfinder MCP server", () => {
  it("initializes, lists seven tools, and serves the spec resource", async () => {
    const bin = fakeCli('echo MOCK');
    const proc2 = startServer(bin);
    try {
      const init = await rpc(proc2, {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05", capabilities: {}, clientInfo: { name: "t", version: "0" } },
      });
      assert.equal(init.result.serverInfo.name, "schema-pathfinder");
      assert.equal(init.result.protocolVersion, "2024-11-05");
      proc2.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
      const tools = await rpc(proc2, { jsonrpc: "2.0", id: 2, method: "tools/list" });
      assert.equal(tools.result.tools.length, 7);
      const names = tools.result.tools.map((tool) => tool.name).sort();
      assert.deepEqual(names, [
        "pathfinder_databases",
        "pathfinder_fixture",
        "pathfinder_help",
        "pathfinder_path",
        "pathfinder_schemas",
        "pathfinder_tables",
        "pathfinder_tree",
      ]);
      for (const tool of tools.result.tools) {
        assert.equal(tool.inputSchema.type, "object");
      }

      const spec = await rpc(proc2, {
        jsonrpc: "2.0",
        id: 3,
        method: "resources/read",
        params: { uri: "pathfinder://spec" },
      });
      assert.match(spec.result.contents[0].text, /Cahier des charges/);

      const prompt = await rpc(proc2, {
        jsonrpc: "2.0",
        id: 4,
        method: "prompts/get",
        params: {
          name: "find_join_path",
          arguments: { from_table: "ClothingItem", to_table: "User" },
        },
      });
      assert.equal(prompt.result.messages[0].role, "user");
      assert.match(prompt.result.messages[0].content.text, /ClothingItem/);
    } finally {
      proc2.kill();
    }
  });

  it("calls pathfinder_path through the CLI wrapper", async () => {
    const bin = fakeCli(`
if [ "$1" = path ]; then
  echo "ClothingItem -> User"
  exit 0
fi
echo unexpected
exit 1
`);
    const proc = startServer(bin);
    try {
      await rpc(proc, {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05", capabilities: {}, clientInfo: { name: "t", version: "0" } },
      });
      const ok = await rpc(proc, {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
          name: "pathfinder_path",
          arguments: {
            from_table: "ClothingItem",
            to_table: "User",
            format: "equation",
            fixture: "fixtures/postgres/dressshot_seed_fk_edges.json",
          },
        },
      });
      assert.equal(ok.result.isError, false);
      assert.match(ok.result.content[0].text, /ClothingItem -> User/);

      const missing = await rpc(proc, {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: { name: "pathfinder_path", arguments: { from_table: "A" } },
      });
      assert.equal(missing.result.isError, true);
      assert.match(missing.result.content[0].text, /USAGE/);

      const dual = await rpc(proc, {
        jsonrpc: "2.0",
        id: 4,
        method: "tools/call",
        params: {
          name: "pathfinder_tables",
          arguments: { fixture: "a.json", sql: "b.sql" },
        },
      });
      assert.equal(dual.result.isError, true);
      assert.match(dual.result.content[0].text, /une seule source/);
    } finally {
      proc.kill();
    }
  });

  it("rejects unknown methods with JSON-RPC -32601", async () => {
    const proc = startServer(fakeCli("echo x"));
    try {
      await rpc(proc, {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05", capabilities: {}, clientInfo: { name: "t", version: "0" } },
      });
      const boom = await rpc(proc, { jsonrpc: "2.0", id: 9, method: "nope/nope" });
      assert.equal(boom.error.code, -32601);
    } finally {
      proc.kill();
    }
  });
});
