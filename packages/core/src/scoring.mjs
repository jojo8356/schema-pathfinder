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
