import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { runCli } from "../packages/cli/src/main.mjs";

function createIo() {
  const output = {
    stdout: "",
    stderr: ""
  };

  return {
    output,
    io: {
      stdout(value) {
        output.stdout += value;
      },
      stderr(value) {
        output.stderr += value;
      }
    }
  };
}

describe("CLI command skeleton", () => {
  it("parses path command and calls the core", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(
      ["path", "ClothingItem", "User", "--fixture", "fixtures/postgres/dressshot_seed_fk_edges.json"],
      io
    );

    assert.equal(exitCode, 0);
    assert.match(output.stdout, /ClothingItem -> ClothingSession -> SellerProfile -> User/);
    assert.equal(output.stderr, "");
  });

  it("exits non-zero when source or target is missing", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(["path", "ClothingItem"], io);

    assert.equal(exitCode, 1);
    assert.match(output.stderr, /missing required argument 'target-table'/);
  });

  it("exits non-zero for unsupported command", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(["inspect", "ClothingItem", "User"], io);

    assert.equal(exitCode, 1);
    assert.match(output.stderr, /unknown command 'inspect'/);
  });

  it("prints equation output with format flag", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(
      [
        "path",
        "ClothingItem",
        "User",
        "--format",
        "equation",
        "--fixture",
        "fixtures/postgres/dressshot_seed_fk_edges.json"
      ],
      io
    );

    assert.equal(exitCode, 0);
    assert.match(output.stdout, /ClothingItem\.clothingSessionId = ClothingSession\.id/);
    assert.match(output.stdout, / -> /);
  });

  it("prints SQL output with format flag", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(
      ["path", "ClothingItem", "User", "--format", "sql", "--fixture", "fixtures/postgres/dressshot_seed_fk_edges.json"],
      io
    );

    assert.equal(exitCode, 0);
    assert.match(output.stdout, /from "ClothingItem" t0/);
    assert.match(output.stdout, /limit 50;/);
  });

  it("exits non-zero for unsupported format", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(
      ["path", "ClothingItem", "User", "--format", "xml", "--fixture", "fixtures/postgres/dressshot_seed_fk_edges.json"],
      io
    );

    assert.equal(exitCode, 1);
    assert.match(output.stderr, /Unsupported format: xml/);
    assert.match(output.stderr, /Supported formats: text, equation, json, sql, mermaid/);
  });

  it("exits non-zero without database config or fixture", async () => {
    const { io, output } = createIo();
    const exitCode = await runCli(["path", "ClothingItem", "User"], io, {});

    assert.equal(exitCode, 1);
    assert.match(output.stderr, /DB_CONFIG_MISSING/);
    assert.doesNotMatch(output.stderr, /postgres:\/\//);
  });
});
