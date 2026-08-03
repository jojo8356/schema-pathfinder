const loopbackAddresses = new Set(["127.0.0.1", "::1", "::ffff:127.0.0.1", "localhost"]);

export function isAdminLocalRequest(request, env = process.env) {
  if (isLoopbackRequest(request)) {
    return true;
  }

  if (env.SCHEMA_PATHFINDER_ADMIN_TOKEN === undefined) {
    return false;
  }

  if (env.SCHEMA_PATHFINDER_ADMIN_TOKEN.length === 0) {
    return false;
  }

  const providedToken = readHeader(request, "x-schema-pathfinder-admin-token");

  if (providedToken === undefined) {
    return false;
  }

  return providedToken === env.SCHEMA_PATHFINDER_ADMIN_TOKEN;
}

export function readHeader(request, headerName) {
  const headers = request.headers;

  if (headers === undefined) {
    return undefined;
  }

  const value = headers[headerName];

  if (Array.isArray(value)) {
    return value[0];
  }

  return value;
}

function isLoopbackRequest(request) {
  const candidates = [request.hostname, request.ip, request.socketRemoteAddress];

  for (const candidate of candidates) {
    if (candidate === undefined) {
      continue;
    }

    if (loopbackAddresses.has(candidate)) {
      return true;
    }
  }

  return false;
}
