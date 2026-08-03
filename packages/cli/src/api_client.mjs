import { createCliError } from "./load_edges.mjs";

export async function fetchPathFromApi(request, fetchImpl = globalThis.fetch) {
  if (fetchImpl === undefined) {
    throw createCliError("API_FETCH_UNAVAILABLE", "global fetch is unavailable in this Node runtime", 1);
  }

  const url = createPathUrl(request.apiUrl);
  const headers = {
    "content-type": "application/json"
  };

  if (request.adminToken !== undefined && request.adminToken.length > 0) {
    headers["x-schema-pathfinder-admin-token"] = request.adminToken;
  }

  const response = await fetchImpl(url.toString(), {
    method: "POST",
    headers,
    body: JSON.stringify({
      sourceTable: request.sourceTable,
      targetTable: request.targetTable,
      format: request.format
    })
  });

  const payload = await readJsonResponse(response);

  if (response.ok === false) {
    throw createCliError(
      "API_REQUEST_FAILED",
      `API request failed with status ${response.status}`,
      1
    );
  }

  return payload;
}

export function createPathUrl(apiUrl) {
  let url;

  try {
    url = new URL(apiUrl);
  } catch (error) {
    throw createCliError("API_URL_INVALID", "API URL must be a valid absolute URL", 1);
  }

  url.pathname = "/admin/pathfinder/path";
  url.search = "";
  url.hash = "";

  return url;
}

async function readJsonResponse(response) {
  try {
    return await response.json();
  } catch (error) {
    throw createCliError("API_RESPONSE_INVALID", "API response must be JSON", 1);
  }
}
