import { outputFormats } from "@schema-pathfinder/core/output_formats";

export class PathRequestDto {
  sourceTable!: string;
  targetTable!: string;
  format!: (typeof outputFormats)[number];
}
