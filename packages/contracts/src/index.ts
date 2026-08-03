export type EvidenceLabel =
  | "declared_fk"
  | "prisma_relation"
  | "name_match"
  | "manual_virtual_edge"
  | "inferred_candidate";

export interface TableIdentifier {
  schema: string;
  table: string;
}

export interface ForeignKeyEdge {
  constraintName: string;
  from: TableIdentifier;
  fromColumn: string;
  to: TableIdentifier;
  toColumn: string;
  evidence: EvidenceLabel[];
}

export interface PathResult {
  source: TableIdentifier;
  target: TableIdentifier;
  score: number;
  length: number;
  evidence: EvidenceLabel[];
  edges: ForeignKeyEdge[];
}

export interface PathfinderResult {
  source: TableIdentifier;
  target: TableIdentifier;
  paths: PathResult[];
  noPathReason?: string;
}
