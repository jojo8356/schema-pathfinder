export const scoringRules = {
  declaredForeignKey: 60,
  extraHopPenalty: -8,
  technicalTablePenalty: -25,
  authSessionPenalty: -15
};

const technicalTableNames = new Set(["_prisma_migrations"]);
const authSessionTableNames = new Set(["Session", "Account", "Verification"]);

export function scorePath(source, target, edges) {
  let score = 0;
  const contributions = [];
  const penalizedTechnicalTables = new Set();
  const penalizedAuthSessionTables = new Set();

  for (const edge of edges) {
    if (edge.evidence.includes("declared_fk")) {
      score += scoringRules.declaredForeignKey;
      contributions.push({
        label: "declared_fk",
        value: scoringRules.declaredForeignKey,
        constraintName: edge.constraintName
      });
    }

    const edgeTables = [edge.from.table, edge.to.table];

    for (const tableName of edgeTables) {
      if (technicalTableNames.has(tableName)) {
        if (penalizedTechnicalTables.has(tableName)) {
          continue;
        }

        penalizedTechnicalTables.add(tableName);
        score += scoringRules.technicalTablePenalty;
        contributions.push({
          label: "technical_table",
          value: scoringRules.technicalTablePenalty,
          table: tableName
        });
      }

      if (authSessionTableNames.has(tableName)) {
        if (penalizedAuthSessionTables.has(tableName)) {
          continue;
        }

        penalizedAuthSessionTables.add(tableName);
        score += scoringRules.authSessionPenalty;
        contributions.push({
          label: "auth_session_table",
          value: scoringRules.authSessionPenalty,
          table: tableName
        });
      }
    }
  }

  if (edges.length > 0) {
    const hopPenalty = edges.length * Math.abs(scoringRules.extraHopPenalty);
    score -= hopPenalty;
    contributions.push({
      label: "extra_hop",
      value: -hopPenalty,
      hops: edges.length
    });
  }

  return {
    source,
    target,
    score,
    length: edges.length,
    evidence: Array.from(new Set(edges.flatMap((edge) => edge.evidence))),
    scoreContributions: contributions,
    edges
  };
}

export function rankPaths(paths) {
  return [...paths]
    .sort((left, right) => {
      if (right.score !== left.score) {
        return right.score - left.score;
      }

      if (left.length !== right.length) {
        return left.length - right.length;
      }

      return left.edges.map((edge) => edge.constraintName).join("|").localeCompare(
        right.edges.map((edge) => edge.constraintName).join("|")
      );
    })
    .slice(0, 3);
}

export const defaultMaxLinks = 5;
export const maxLinksCeiling = 8;
export const maxRankedPaths = 50;

export function clampMaxLinks(maxLinks) {
  if (Number.isFinite(maxLinks) === false) {
    return defaultMaxLinks;
  }

  const rounded = Math.trunc(maxLinks);

  if (rounded < 1) {
    return 1;
  }

  if (rounded > maxLinksCeiling) {
    return maxLinksCeiling;
  }

  return rounded;
}

// Order paths by increasing complexity: fewest links first, then the
// higher-scoring path within the same length, then a stable tie-break on the
// constraint names. This lets the UI list several join options so the user can
// pick one that avoids tables which are not yet populated during data entry.
export function rankPathsByComplexity(paths) {
  return [...paths]
    .sort((left, right) => {
      if (left.length !== right.length) {
        return left.length - right.length;
      }

      if (right.score !== left.score) {
        return right.score - left.score;
      }

      return left.edges.map((edge) => edge.constraintName).join("|").localeCompare(
        right.edges.map((edge) => edge.constraintName).join("|")
      );
    })
    .slice(0, maxRankedPaths);
}
