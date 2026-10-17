import { escapeHtml } from "./code.js";
import { fallbackDescription, uniqueSeoTitles } from "./seo_titles.js";
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
import {
  articleStructuredData,
  renderSeo,
  softwareStructuredData,
  type BreadcrumbCrumb,
} from "./seo.js";
import { DOCS_STYLES, docsThemeScript } from "./theme.js";

/** Absolute path of the raster social-preview image (1200×630 PNG). */
const OG_IMAGE_PATH = "/og.png";

/**
 * Deterministic fallback content date for pages that do not carry an explicit
 * `publishedAt`/`updatedAt`. Kept as a constant (not `Date.now()`) so the emitted
 * tree stays byte-identical across renders.
 */
const DEFAULT_CONTENT_DATE = "2026-09-14";

export interface DocumentationPageSource {
  readonly path: string;
  readonly source: string;
  readonly section: string;
  readonly updatedAt?: string;
  readonly publishedAt?: string;
  /** When true the page emits TechArticle + BreadcrumbList JSON-LD. */
  readonly article?: boolean;
}

export interface CompiledPage {
  readonly path: string;
  readonly title: string;
  readonly html: string;
  readonly document: MarkdownDocument;
  /** W3C date used for the page's `<lastmod>` sitemap entry. */
  readonly lastmod: string;
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

/** Title-case a URL path segment the same way the navigation builder labels sections. */
function segmentLabel(segment: string): string {
  return segment.replaceAll("-", " ").replace(/\b\w/gu, (letter) => letter.toUpperCase());
}

/** Whether a page should be described as a content article (TechArticle + breadcrumbs). */
function isArticlePage(source: DocumentationPageSource): boolean {
  return source.article === true || /^\/docs\/(?:methods|guides)\/.+/u.test(source.path);
}

/**
 * Breadcrumb trail from the site root to the current page, deriving intermediate
 * labels from the URL segments and using the resolved page title for the leaf.
 */
function breadcrumbsFor(
  path: string,
  title: string,
  origin: string,
  canonical: string,
): readonly BreadcrumbCrumb[] {
  const base = origin.replace(/\/$/, "");
  const crumbs: BreadcrumbCrumb[] = [{ name: "Home", url: `${base}/` }];
  const segments = path.split("/").filter(Boolean);
  let accumulated = "";
  segments.forEach((segment, index) => {
    accumulated += `/${segment}`;
    const isLeaf = index === segments.length - 1;
    crumbs.push({
      name: isLeaf ? title : segmentLabel(segment),
      url: isLeaf ? canonical : `${base}${accumulated}`,
    });
  });
  return crumbs;
}

function renderPage(
  page: ParsedPage,
  navigation: readonly NavigationSection[],
  configuration: SiteConfiguration,
  seoTitle: string = page.title,
): string {
  const { document, source, title } = page;
  const description = document.metadata.description ?? fallbackDescription(title, document.plainText);
  const canonical = new URL(
    document.metadata.canonical ?? source.path,
    configuration.origin,
  ).toString();
  const adjacent = adjacentPages(navigation, source.path);
  const productName = configuration.name ?? "LawSynth";
  const version = configuration.version
    ? `<span>${escapeHtml(configuration.version)}</span>`
    : "";
  const article = isArticlePage(source);
  const publishedAt = source.publishedAt ?? (article ? DEFAULT_CONTENT_DATE : undefined);
  const modifiedAt = source.updatedAt ?? source.publishedAt ?? (article ? DEFAULT_CONTENT_DATE : undefined);
  const seo = renderSeo(
    {
      title: seoTitle,
      description,
      canonicalUrl: canonical,
      imageUrl: OG_IMAGE_PATH,
      ...(article ? { type: "article" as const } : {}),
      ...(publishedAt === undefined ? {} : { publishedAt }),
      ...(modifiedAt === undefined ? {} : { modifiedAt }),
    },
    productName,
  );
  // Content pages get page-specific TechArticle + BreadcrumbList JSON-LD; every
  // other page keeps the generic SoftwareApplication graph. Both helpers escape
  // `<`, so the output is safe to inline inside a <script> element.
  const structuredData = article
    ? `<script type="application/ld+json">${articleStructuredData({
        headline: title,
        description,
        url: canonical,
        origin: configuration.origin,
        imageUrl: new URL(OG_IMAGE_PATH, configuration.origin).toString(),
        breadcrumbs: breadcrumbsFor(source.path, title, configuration.origin, canonical),
        ...(publishedAt === undefined ? {} : { publishedAt }),
        ...(modifiedAt === undefined ? {} : { modifiedAt }),
      })}</script>`
    : `<script type="application/ld+json">${softwareStructuredData(
        productName,
        configuration.version ?? "0.1.0",
        configuration.origin,
      )}</script>`;

  return [
    "<!doctype html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="utf-8">',
    '<meta name="viewport" content="width=device-width,initial-scale=1">',
    '<link rel="icon" href="/favicon.svg" type="image/svg+xml">',
    '<link rel="apple-touch-icon" href="/favicon.svg">',
    '<meta name="theme-color" content="#18201d">',
    seo,
    structuredData,
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
      return `  <url><loc>${location}</loc><lastmod>${escapeHtml(page.lastmod)}</lastmod></url>`;
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
  const seoTitles = uniqueSeoTitles(parsed.map((page) => ({ path: page.source.path, title: page.title })));
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
      html: renderPage(page, navigation, configuration, seoTitles.get(page.source.path) ?? page.title),
      document: page.document,
      lastmod: page.source.updatedAt ?? page.source.publishedAt ?? DEFAULT_CONTENT_DATE,
    });
  });

  return Object.freeze({
    pages: Object.freeze(pages),
    navigation,
    search,
    sitemap: createSitemap(pages, origin),
  });
}
