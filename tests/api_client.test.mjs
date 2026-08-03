import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createPathUrl, fetchPathFromApi } from "@schema-pathfinder/cli/api_client";

describe("CLI API sync client", () => {
  it("normalizes API endpoint URLs", () => {
    assert.equal(
      createPathUrl("https://schema-pathfinder.example.com/base?debug=1").toString(),
      "https://schema-pathfinder.example.com/admin/pathfinder/path"
    );
  });

  it("posts path requests with the admin token header", async () => {
    const calls = [];
    const fakeFetch = async (url, options) => {
      calls.push({ url, options });

      return {
        ok: true,
        status: 200,
        async json() {
          return {
            rendered: "A.id = B.aId",
            paths: []
          };
        }
      };
    };

    const result = await fetchPathFromApi(
      {
        apiUrl: "https://api.example.test",
        adminToken: "admin-token",
        sourceTable: "A",
        targetTable: "B",
        format: "equation"
      },
      fakeFetch
    );

    assert.equal(result.rendered, "A.id = B.aId");
    assert.equal(calls.length, 1);
    assert.equal(calls[0].url, "https://api.example.test/admin/pathfinder/path");
    assert.equal(calls[0].options.method, "POST");
    assert.equal(calls[0].options.headers["x-schema-pathfinder-admin-token"], "admin-token");
    assert.deepEqual(JSON.parse(calls[0].options.body), {
      sourceTable: "A",
      targetTable: "B",
      format: "equation"
    });
  });

  it("returns stable CLI errors for invalid URLs", async () => {
    await assert.rejects(
      () => fetchPathFromApi({
        apiUrl: "not a url",
        sourceTable: "A",
        targetTable: "B",
        format: "text"
      }),
      (error) => {
        assert.equal(error.code, "API_URL_INVALID");
        assert.equal(error.exitCode, 1);
        return true;
      }
    );
  });
});
