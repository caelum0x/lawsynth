export interface NavigationPage {
  readonly path: string;
  readonly title: string;
  readonly section: string;
  readonly order?: number;
  readonly hidden?: boolean;
}

export interface NavigationSection {
  readonly id: string;
  readonly title: string;
  readonly pages: readonly NavigationPage[];
}

function safePath(path: string): void {
  if (!path.startsWith("/") || path.includes("..") || /[\r\n\0]/u.test(path)) throw new RangeError(`invalid navigation path: ${path}`);
}

export function buildNavigation(pages: readonly NavigationPage[], sectionTitles: Readonly<Record<string, string>> = {}): readonly NavigationSection[] {
  const paths = new Set<string>();
  const sections = new Map<string, NavigationPage[]>();
  for (const page of pages) {
    safePath(page.path);
    if (!page.title.trim() || !page.section.trim() || paths.has(page.path)) throw new RangeError(`invalid or duplicate navigation page: ${page.path}`);
    paths.add(page.path);
    if (page.hidden === true) continue;
    sections.set(page.section, [...(sections.get(page.section) ?? []), Object.freeze(page)]);
  }
  return Object.freeze([...sections].map(([id, entries]) => Object.freeze({
    id,
    title: sectionTitles[id] ?? id.replaceAll("-", " ").replace(/\b\w/gu, (letter) => letter.toUpperCase()),
    pages: Object.freeze(entries.sort((left, right) => (left.order ?? Number.MAX_SAFE_INTEGER) - (right.order ?? Number.MAX_SAFE_INTEGER) || left.title.localeCompare(right.title))),
  })));
}

export function adjacentPages(navigation: readonly NavigationSection[], path: string): { readonly previous?: NavigationPage; readonly next?: NavigationPage; } {
  const pages = navigation.flatMap((section) => section.pages);
  const index = pages.findIndex((page) => page.path === path);
  if (index < 0) return {};
  const previous = pages[index - 1];
  const next = pages[index + 1];
  return { ...(previous === undefined ? {} : { previous }), ...(next === undefined ? {} : { next }) };
}
