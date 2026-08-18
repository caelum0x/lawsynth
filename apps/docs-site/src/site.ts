import { escapeHtml } from "./code.js";
import {
  parseMarkdown,
  renderMarkdown,
  type MarkdownDocument,
} from "./markdown.js";
import {
  adjacentPages,
  buildNavigation,
  type NavigationPage,
  type NavigationSection,
} from "./navigation.js";
import { SearchIndex } from "./search.js";
import { renderSeo } from "./seo.js";
import { DOCS_STYLES, docsThemeScript } from "./theme.js";

export interface DocumentationPageSource {
  readonly path: string;
  readonly source: string;
  readonly section: string;
  readonly updatedAt?: string;
}

export interface CompiledPage {
  readonly path: string;
  readonly title: string;
  readonly html: string;
  readonly document: MarkdownDocument;
}

export interface DocumentationSite {
  readonly pages: readonly CompiledPage[];
  readonly navigation: readonly NavigationSection[];
  readonly search: SearchIndex;
  readonly sitemap: string;
}

export interface SiteConfiguration {
  readonly origin: string;
  readonly name?: string;
  readonly version?: string;
  readonly includeDrafts?: boolean;
}

interface ParsedPage {
  readonly source: DocumentationPageSource;
  readonly document: MarkdownDocument;
  readonly title: string;
}

function validateOrigin(value: string): URL {
  const origin = new URL(value);
  const localDevelopment =
    origin.hostname === "localhost" || origin.hostname === "127.0.0.1";

  if (origin.protocol !== "https:" && !localDevelopment) {
    throw new RangeError("documentation origin must use HTTPS");
  }
  if (origin.username || origin.password || origin.search || origin.hash) {
    throw new RangeError("documentation origin must not contain credentials or a query");
  }
  return origin;
}

function resolvePageTitle(
  source: DocumentationPageSource,
  document: MarkdownDocument,
): string {
  const title = document.metadata.title ?? document.headings[0]?.text;
  if (!title) {
    throw new RangeError(`documentation page ${source.path} has no title`);
  }
  return title;
}

function parseSources(
  sources: readonly DocumentationPageSource[],
  includeDrafts: boolean,
): readonly ParsedPage[] {
  const seenPaths = new Set<string>();

  return sources.flatMap((source) => {
    if (!source.path.startsWith("/") || source.path.includes("..")) {
      throw new RangeError(`documentation path is unsafe: ${source.path}`);
    }
    if (seenPaths.has(source.path)) {
      throw new RangeError(`duplicate documentation path: ${source.path}`);
    }
    seenPaths.add(source.path);

    const document = parseMarkdown(source.source);
    if (document.metadata.draft === true && !includeDrafts) {
      return [];
    }

    return [{ source, document, title: resolvePageTitle(source, document) }];
  });
}

function navigationInput(page: ParsedPage): NavigationPage {
  return {
    path: page.source.path,
    title: page.title,
    section: page.source.section,
    ...(page.document.metadata.order === undefined
      ? {}
      : { order: page.document.metadata.order }),
  };
}

function renderNavigation(
  navigation: readonly NavigationSection[],
  currentPath: string,
): string {
  return navigation
    .map((section) => {
      const pages = section.pages
        .map((page) => {
          const current =
            page.path === currentPath ? ' aria-current="page"' : "";
          return `<li><a href="${escapeHtml(page.path)}"${current}>${escapeHtml(page.title)}</a></li>`;
        })
        .join("");

      return [
        "<section>",
        `<h2>${escapeHtml(section.title)}</h2>`,
        `<ul>${pages}</ul>`,
        "</section>",
      ].join("");
    })
    .join("");
}

function renderPagination(
  adjacent: ReturnType<typeof adjacentPages>,
): string {
  const previous = adjacent.previous
    ? `<a rel="prev" href="${escapeHtml(adjacent.previous.path)}">← ${escapeHtml(adjacent.previous.title)}</a>`
    : "<span></span>";
  const next = adjacent.next
    ? `<a rel="next" href="${escapeHtml(adjacent.next.path)}">${escapeHtml(adjacent.next.title)} →</a>`
    : "";

  return `<nav class="pagination" aria-label="Adjacent pages">${previous}${next}</nav>`;
}

function renderPage(
  page: ParsedPage,
  navigation: readonly NavigationSection[],
  configuration: SiteConfiguration,
): string {
  const { document, source, title } = page;
  const description =
    document.metadata.description ?? document.plainText.slice(0, 180);
  const canonical = new URL(
    document.metadata.canonical ?? source.path,
    configuration.origin,
  ).toString();
  const adjacent = adjacentPages(navigation, source.path);
  const productName = configuration.name ?? "LawSynth";
  const version = configuration.version
    ? `<span>${escapeHtml(configuration.version)}</span>`
    : "";
  const seo = renderSeo(
    {
      title,
      description,
      canonicalUrl: canonical,
      ...(source.updatedAt === undefined
        ? {}
        : { modifiedAt: source.updatedAt }),
    },
    productName,
  );

  return [
    "<!doctype html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width,initial-scale=1">',
    seo,
    `<style>${DOCS_STYLES}</style>`,
    `<script>${docsThemeScript()}</script>`,
    "</head>",
    "<body>",
    '<a href="#content" class="skip-link">Skip to content</a>',
    `<header><strong>${escapeHtml(productName)}</strong>${version}` +
      '<a class="docs-repo" href="https://github.com/caelum0x/lawsynth" rel="noopener noreferrer">GitHub</a>' +
      "</header>",
    '<div class="docs-shell">',
    `<aside><nav aria-label="Documentation">${renderNavigation(navigation, source.path)}</nav></aside>`,
    '<main id="content">',
    `<article>${renderMarkdown(document)}</article>`,
    renderPagination(adjacent),
    "</main>",
    "</div>",
    '<footer class="docs-footer">' +
      '<nav aria-label="Site links">' +
      '<a href="/getting-started">Docs</a>' +
      '<a href="/reference/cli">CLI reference</a>' +
      '<a href="https://github.com/caelum0x/lawsynth" rel="noopener noreferrer">GitHub</a>' +
      '<a href="mailto:caelum0x42@gmail.com">Contact</a>' +
      "</nav>" +
      `<p>${escapeHtml(productName)} — deterministic discovery of executable mathematical worlds. ` +
      'Questions or collaboration: <a href="mailto:caelum0x42@gmail.com">caelum0x42@gmail.com</a>.</p>' +
      "</footer>",
    "</body>",
    "</html>",
  ].join("");
}

function createSitemap(
  pages: readonly CompiledPage[],
  origin: URL,
): string {
  const urls = pages
    .map((page) => {
      const location = escapeHtml(new URL(page.path, origin).toString());
      return `  <url><loc>${location}</loc></url>`;
    })
    .join("\n");

  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    urls,
    "</urlset>",
  ].join("\n");
}

export function compileSite(
  sources: readonly DocumentationPageSource[],
  configuration: SiteConfiguration,
): DocumentationSite {
  const origin = validateOrigin(configuration.origin);
  const parsed = parseSources(
    sources,
    configuration.includeDrafts === true,
  );
  const navigation = buildNavigation(parsed.map(navigationInput));
  const search = new SearchIndex();

  const pages = parsed.map((page, index): CompiledPage => {
    search.add({
      id: `page-${index}`,
      path: page.source.path,
      title: page.title,
      text: page.document.plainText,
      headings: page.document.headings.map((heading) => heading.text),
      ...(page.document.metadata.tags === undefined
        ? {}
        : { tags: page.document.metadata.tags }),
      ...(configuration.version === undefined
        ? {}
        : { version: configuration.version }),
    });

    return Object.freeze({
      path: page.source.path,
      title: page.title,
      html: renderPage(page, navigation, configuration),
      document: page.document,
    });
  });

  return Object.freeze({
    pages: Object.freeze(pages),
    navigation,
    search,
    sitemap: createSitemap(pages, origin),
  });
}
