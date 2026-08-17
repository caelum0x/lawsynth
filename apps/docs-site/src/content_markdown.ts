/**
 * Tiny helpers for authoring the documentation content that this site renders.
 * They emit ordinary Markdown strings so the content flows through the existing
 * `parseMarkdown` / `compileSite` pipeline unchanged — every fenced block is
 * still parsed by `markdown.ts` and highlighted by the `code.ts` block builder.
 */

export interface FrontMatterFields {
  readonly title: string;
  readonly description: string;
  readonly order?: number;
  readonly tags?: readonly string[];
}

/** Renders YAML-style front matter understood by `parseMetadata`. */
export function frontMatter(fields: FrontMatterFields): string {
  const lines = [`title: ${fields.title}`, `description: ${fields.description}`];
  if (fields.order !== undefined) lines.push(`order: ${fields.order}`);
  if (fields.tags !== undefined && fields.tags.length > 0) lines.push(`tags: ${fields.tags.join(", ")}`);
  return ["---", ...lines, "---"].join("\n");
}

/** Renders a fenced code block. `source` is emitted verbatim; the Markdown parser escapes it. */
export function codeFence(language: string, source: string): string {
  return "```" + language + "\n" + source + "\n" + "```";
}

/** Joins document sections with the blank lines the block parser expects. */
export function markdownDocument(...sections: readonly string[]): string {
  return sections.join("\n\n") + "\n";
}
