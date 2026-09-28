import { z } from "zod";

export const outputFormats = ["text", "equation", "json", "sql", "mermaid"];

export const outputFormatSchema = z.enum(outputFormats);

export function parseOutputFormat(value) {
  return outputFormatSchema.safeParse(value);
}

export function formatList() {
  return outputFormats.join(", ");
}
