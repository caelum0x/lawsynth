/**
 * Search engines treat identical <title> tags as duplicate pages. Several docs
 * pages legitimately share a heading (e.g. "/systems/lorenz" and
 * "/examples/lorenz"), so the SEO title of every non-primary duplicate gets a
 * short path qualifier. The visible H1 is left unchanged.
 */
export interface TitledPath { readonly path: string; readonly title: string; }

function label(segment: string): string {
  return segment.replaceAll("-", " ").replace(/\b\w/gu, (letter) => letter.toUpperCase());
}

function qualifier(path: string): string {
  const parents = path.split("/").filter(Boolean).slice(0, -1);
  if (parents[0] === "docs") return parents.length > 1 ? parents.slice(1).map(label).join(" / ") : "Docs";
  return parents.map(label).join(" / ");
}

function primaryRank(path: string): readonly [number, number, string] {
  const depth = path.split("/").filter(Boolean).length;
  return [depth, path.startsWith("/systems/") ? 0 : 1, path];
}

function comparePrimary(left: string, right: string): number {
  const a = primaryRank(left);
  const b = primaryRank(right);
  return a[0] - b[0] || a[1] - b[1] || a[2].localeCompare(b[2]);
}

export function uniqueSeoTitles(pages: readonly TitledPath[]): ReadonlyMap<string, string> {
  const groups = new Map<string, string[]>();
  for (const page of pages) groups.set(page.title, [...(groups.get(page.title) ?? []), page.path]);
  const result = new Map<string, string>();
  for (const [title, paths] of groups) {
    const [primary, ...rest] = [...paths].sort(comparePrimary);
    if (primary !== undefined) result.set(primary, title);
    for (const path of rest) {
      const extra = qualifier(path);
      result.set(path, extra ? `${title} (${extra})` : `${title} (${path})`);
    }
  }
  return result;
}

/** Meta description fallback when a page has no frontmatter description. */
export function fallbackDescription(title: string, plainText: string): string {
  const text = plainText.replace(/\s+/g, " ").trim();
  const withoutHeading = text.startsWith(`${title} `) ? text.slice(title.length + 1) : text;
  return withoutHeading.replace(/\*\*|__|`/g, "").replace(/(^|\s)[*_](\S)/g, "$1$2").trim();
}
