import { readFileSync } from "node:fs";

const files = [
  "fixtures/postgres/dressshot_seed_fk_edges.json",
  "fixtures/postgres/dressshot_expected_clothing_item_user.json",
  "fixtures/postgres/dressshot_expected_no_path.json"
];

for (const file of files) {
  JSON.parse(readFileSync(file, "utf8"));
}

const expectedPath = JSON.parse(
  readFileSync("fixtures/postgres/dressshot_expected_clothing_item_user.json", "utf8")
);

const rendererNames = ["text", "equation", "sql", "mermaid"];
const renderers = expectedPath.paths[0].renderers;
const missingRenderers = rendererNames.filter((name) => typeof renderers[name] !== "string");

if (missingRenderers.length > 0) {
  console.error(`Missing contract renderers: ${missingRenderers.join(", ")}`);
  process.exit(1);
}

console.log("schema-pathfinder contract fixtures ok");
