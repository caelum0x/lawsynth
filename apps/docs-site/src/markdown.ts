import { codeBlock, escapeHtml, type CodeBlock } from "./code.js";

export interface DocumentMetadata {
  readonly title?: string;
  readonly description?: string;
  readonly order?: number;
  readonly draft?: boolean;
  readonly tags?: readonly string[];
  readonly canonical?: string;
}

export type MarkdownBlock =
  | {
      readonly kind: "heading";
      readonly level: number;
      readonly text: string;
      readonly id: string;
    }
  | { readonly kind: "paragraph"; readonly text: string }
  | { readonly kind: "code"; readonly block: CodeBlock }
  | {
      readonly kind: "list";
      readonly ordered: boolean;
      readonly items: readonly string[];
    }
  | { readonly kind: "quote"; readonly text: string }
  | { readonly kind: "rule" };

export interface MarkdownDocument {
  readonly metadata: DocumentMetadata;
  readonly blocks: readonly MarkdownBlock[];
  readonly headings: readonly Extract<
    MarkdownBlock,
    { kind: "heading" }
  >[];
  readonly plainText: string;
}

interface ParsedSource {
  readonly metadata: DocumentMetadata;
  readonly body: readonly string[];
}

function slug(value: string): string {
  return (
    value
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[^a-z0-9\s-]/gu, "")
      .trim()
      .replace(/\s+/gu, "-")
      .replace(/-+/gu, "-") || "section"
  );
}

function parseBoolean(value: string | undefined): boolean | undefined {
  if (value === undefined) return undefined;
  if (value === "true") return true;
  if (value === "false") return false;
  throw new TypeError("front matter draft must be true or false");
}

function parseOrder(value: string | undefined): number | undefined {
  if (value === undefined) return undefined;

  const order = Number(value);
  if (!Number.isFinite(order)) {
    throw new TypeError("front matter order must be numeric");
  }
  return order;
}

function parseMetadata(lines: readonly string[]): ParsedSource {
  if (lines[0] !== "---") {
    return { metadata: {}, body: lines };
  }

  const end = lines.indexOf("---", 1);
  if (end < 0) {
    throw new SyntaxError("unterminated Markdown front matter");
  }

  const fields = new Map<string, string>();
  for (const line of lines.slice(1, end)) {
    const separator = line.indexOf(":");
    if (separator < 1) {
      throw new SyntaxError(`invalid front matter line: ${line}`);
    }

    const key = line.slice(0, separator).trim().toLowerCase();
    const value = line.slice(separator + 1).trim();
    if (fields.has(key)) {
      throw new SyntaxError(`duplicate front matter field: ${key}`);
    }
    fields.set(key, value);
  }

  const supportedFields = new Set([
    "title",
    "description",
    "order",
    "draft",
    "tags",
    "canonical",
  ]);
  for (const key of fields.keys()) {
    if (!supportedFields.has(key)) {
      throw new SyntaxError(`unsupported front matter field: ${key}`);
    }
  }

  const tags = fields
    .get("tags")
    ?.split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
  const order = parseOrder(fields.get("order"));
  const draft = parseBoolean(fields.get("draft"));
  const title = fields.get("title");
  const description = fields.get("description");
  const canonical = fields.get("canonical");

  return {
    metadata: {
      ...(title ? { title } : {}),
      ...(description ? { description } : {}),
      ...(order === undefined ? {} : { order }),
      ...(draft === undefined ? {} : { draft }),
      ...(tags === undefined ? {} : { tags: Object.freeze(tags) }),
      ...(canonical ? { canonical } : {}),
    },
    body: lines.slice(end + 1),
  };
}

function readCodeBlock(
  lines: readonly string[],
  start: number,
): { readonly block: MarkdownBlock; readonly next: number } {
  const language = lines[start]!.slice(3).trim();
  const content: string[] = [];
  let index = start + 1;

  while (index < lines.length && !lines[index]!.startsWith("```")) {
    content.push(lines[index]!);
    index += 1;
  }
  if (index >= lines.length) {
    throw new SyntaxError("unterminated code fence");
  }

  return {
    block: { kind: "code", block: codeBlock(content.join("\n"), language) },
    next: index + 1,
  };
}

function readQuote(
  lines: readonly string[],
  start: number,
): { readonly block: MarkdownBlock; readonly next: number } {
  const values: string[] = [];
  let index = start;

  while (lines[index]?.startsWith("> ")) {
    values.push(lines[index]!.slice(2));
    index += 1;
  }

  return {
    block: { kind: "quote", text: values.join(" ") },
    next: index,
  };
}

const LIST_ITEM = /^\s*(?:([-*+])|(\d+)\.)\s+(.+)$/u;

function readList(
  lines: readonly string[],
  start: number,
): { readonly block: MarkdownBlock; readonly next: number } {
  const first = LIST_ITEM.exec(lines[start]!);
  if (!first) {
    throw new TypeError("list reader requires a list item");
  }

  const ordered = first[2] !== undefined;
  const items: string[] = [];
  let index = start;

  while (index < lines.length) {
    const item = LIST_ITEM.exec(lines[index]!);
    if (!item || (item[2] !== undefined) !== ordered) break;
    items.push(item[3]!);
    index += 1;
  }

  return {
    block: { kind: "list", ordered, items: Object.freeze(items) },
    next: index,
  };
}

function startsBlock(line: string): boolean {
  return (
    /^(#{1,6})\s/u.test(line) ||
    line.startsWith("```") ||
    line.startsWith("> ") ||
    LIST_ITEM.test(line) ||
    /^ {0,3}([-*_])(?:\s*\1){2,}\s*$/u.test(line)
  );
}

function readParagraph(
  lines: readonly string[],
  start: number,
): { readonly block: MarkdownBlock; readonly next: number } {
  const content = [lines[start]!.trim()];
  let index = start + 1;

  while (
    index < lines.length &&
    lines[index]!.trim().length > 0 &&
    !startsBlock(lines[index]!)
  ) {
    content.push(lines[index]!.trim());
    index += 1;
  }

  return {
    block: { kind: "paragraph", text: content.join(" ") },
    next: index,
  };
}

function plainTextFor(block: MarkdownBlock): readonly string[] {
  switch (block.kind) {
    case "code":
    case "rule":
      return [];
    case "list":
      return block.items;
    case "heading":
    case "paragraph":
    case "quote":
      return [block.text];
  }
}

export function parseMarkdown(source: string): MarkdownDocument {
  const parsed = parseMetadata(source.replaceAll("\r\n", "\n").split("\n"));
  const blocks: MarkdownBlock[] = [];
  const headingIds = new Map<string, number>();
  let index = 0;

  while (index < parsed.body.length) {
    const line = parsed.body[index]!;
    if (!line.trim()) {
      index += 1;
      continue;
    }

    if (line.startsWith("```")) {
      const result = readCodeBlock(parsed.body, index);
      blocks.push(result.block);
      index = result.next;
      continue;
    }

    const heading = /^(#{1,6})\s+(.+)$/u.exec(line);
    if (heading) {
      const text = heading[2]!.trim();
      const baseId = slug(text);
      const occurrence = headingIds.get(baseId) ?? 0;
      headingIds.set(baseId, occurrence + 1);
      blocks.push({
        kind: "heading",
        level: heading[1]!.length,
        text,
        id: occurrence === 0 ? baseId : `${baseId}-${occurrence + 1}`,
      });
      index += 1;
      continue;
    }

    if (/^ {0,3}([-*_])(?:\s*\1){2,}\s*$/u.test(line)) {
      blocks.push({ kind: "rule" });
      index += 1;
      continue;
    }

    if (line.startsWith("> ")) {
      const result = readQuote(parsed.body, index);
      blocks.push(result.block);
      index = result.next;
      continue;
    }

    if (LIST_ITEM.test(line)) {
      const result = readList(parsed.body, index);
      blocks.push(result.block);
      index = result.next;
      continue;
    }

    const result = readParagraph(parsed.body, index);
    blocks.push(result.block);
    index = result.next;
  }

  const headings = blocks.filter(
    (block): block is Extract<MarkdownBlock, { kind: "heading" }> =>
      block.kind === "heading",
  );
  const plainText = blocks.flatMap(plainTextFor).join("\n");

  return Object.freeze({
    metadata: Object.freeze(parsed.metadata),
    blocks: Object.freeze(blocks),
    headings: Object.freeze(headings),
    plainText,
  });
}

function inline(text: string): string {
  return escapeHtml(text)
    .replace(/`([^`]+)`/gu, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/gu, "<strong>$1</strong>")
    .replace(
      /\[([^\]]+)\]\(([^)]+)\)/gu,
      (_match, label: string, target: string) =>
        /^(?:https?:\/\/|\/|#|\.\.?\/)/u.test(target)
          ? `<a href="${escapeHtml(target)}">${label}</a>`
          : label,
    );
}

function renderBlock(block: MarkdownBlock): string {
  switch (block.kind) {
    case "heading":
      return `<h${block.level} id="${block.id}">${inline(block.text)}</h${block.level}>`;
    case "paragraph":
      return `<p>${inline(block.text)}</p>`;
    case "quote":
      return `<blockquote><p>${inline(block.text)}</p></blockquote>`;
    case "rule":
      return "<hr>";
    case "list": {
      const tag = block.ordered ? "ol" : "ul";
      const items = block.items
        .map((item) => `<li>${inline(item)}</li>`)
        .join("");
      return `<${tag}>${items}</${tag}>`;
    }
    case "code":
      return `<pre data-language="${block.block.language}"><code>${block.block.highlightedHtml}</code></pre>`;
  }
}

export function renderMarkdown(document: MarkdownDocument): string {
  return document.blocks.map(renderBlock).join("\n");
}
