import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  discoverPostgresForeignKeys,
  mapPostgresForeignKeyRow,
  postgresForeignKeyMetadataSql
} from "../packages/core/src/postgres_fk_metadata.mjs";

describe("PostgreSQL FK metadata discovery", () => {
  it("uses catalog tables only and does not query application rows", () => {
    assert.match(postgresForeignKeyMetadataSql, /pg_constraint/);
    assert.match(postgresForeignKeyMetadataSql, /pg_attribute/);
    assert.doesNotMatch(postgresForeignKeyMetadataSql, /select\s+\*\s+from\s+"?ClothingItem"?/i);
    assert.doesNotMatch(postgresForeignKeyMetadataSql, /count\s*\(/i);
    assert.doesNotMatch(postgresForeignKeyMetadataSql, /limit\s+\d+/i);
  });

  it("maps pg_constraint rows to FK edges", () => {
    const edge = mapPostgresForeignKeyRow({
      source_schema: "public",
      source_table: "ClothingItem",
      source_column: "clothingSessionId",
      target_schema: "public",
      target_table: "ClothingSession",
      target_column: "id",
      constraint_name: "ClothingItem_clothingSessionId_fkey"
    });

    assert.deepEqual(edge, {
      constraintName: "ClothingItem_clothingSessionId_fkey",
      from: {
        schema: "public",
        table: "ClothingItem"
      },
      fromColumn: "clothingSessionId",
      to: {
        schema: "public",
        table: "ClothingSession"
      },
      toColumn: "id",
      evidence: ["declared_fk"]
    });
  });

  it("discovers FK edges through a PostgreSQL-compatible client", async () => {
    const calls = [];
    const client = {
      async query(sql, params) {
        calls.push({ sql, params });

        return {
          rows: [
            {
              source_schema: "public",
              source_table: "SellerProfile",
              source_column: "userId",
              target_schema: "public",
              target_table: "User",
              target_column: "id",
              constraint_name: "SellerProfile_userId_fkey"
            }
          ]
        };
      }
    };

    const edges = await discoverPostgresForeignKeys(client, ["public"]);

    assert.equal(calls.length, 1);
    assert.deepEqual(calls[0].params, [["public"]]);
    assert.equal(edges[0].constraintName, "SellerProfile_userId_fkey");
    assert.equal(edges[0].evidence.includes("declared_fk"), true);
  });
});
