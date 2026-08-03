import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { isAdminLocalRequest } from "@schema-pathfinder/api/admin_local_guard";

describe("API admin/local guard", () => {
  it("allows loopback requests", () => {
    assert.equal(isAdminLocalRequest({ ip: "127.0.0.1" }, {}), true);
    assert.equal(isAdminLocalRequest({ hostname: "localhost" }, {}), true);
  });

  it("allows matching admin token", () => {
    const request = {
      headers: {
        "x-schema-pathfinder-admin-token": "secret-token"
      },
      ip: "203.0.113.20"
    };

    assert.equal(
      isAdminLocalRequest(request, { SCHEMA_PATHFINDER_ADMIN_TOKEN: "secret-token" }),
      true
    );
  });

  it("rejects missing or invalid token for non-local request", () => {
    const request = {
      headers: {
        "x-schema-pathfinder-admin-token": "wrong"
      },
      ip: "203.0.113.20"
    };

    assert.equal(isAdminLocalRequest(request, { SCHEMA_PATHFINDER_ADMIN_TOKEN: "secret-token" }), false);
    assert.equal(isAdminLocalRequest({ ip: "203.0.113.20" }, {}), false);
  });
});
