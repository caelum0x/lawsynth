// Static-site emitter for Cloudflare Pages (lawsynth.dev).
//
// The documentation SSG (`./dist/src/content.js`, built from TypeScript) is
// deliberately Node-free and produces an in-memory `DocumentationSite`. This
// thin ESM script is the deployment boundary: it imports that tested,
// deterministic model and writes it to a directory of plain files. It lives as
// `.mjs` (not `.ts`) so it can use Node's `fs`/`path`/`process` without adding
// `@types/node` to the pure-TypeScript site build.
//
// Every page is already a full, self-contained HTML document (inline styles +
// theme script), so emission is a pure file write and the output tree is
// byte-identical across runs.
//
// Usage (after `npm run build`): node render-static.mjs [outputDir=public]

import { copyFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { docsContentPages } from "./dist/src/content.js";
import { compileSite } from "./dist/src/site.js";

import { loadDocsSources } from "./load-docs.mjs";

/** Static assets (favicon, og image) copied verbatim into the output tree. */
const ASSETS_DIR = fileURLToPath(new URL("./assets", import.meta.url));

/** Production configuration for https://lawsynth.dev. */
export const SITE_CONFIGURATION = Object.freeze({ origin: "https://lawsynth.dev", name: "LawSynth" });

// `.dev` is HSTS-preloaded (HTTPS enforced by the TLD), so we add the remaining
// hardening headers plus a CSP that permits the pages' own inline style/script.
const HEADERS = `/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: strict-origin-when-cross-origin
  Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; img-src 'self' data:; base-uri 'self'; form-action 'self'
`;

const REDIRECTS = `# Cloudflare Pages redirects. Add canonical/legacy path rules here.
`;

function robotsTxt(origin) {
  const base = origin.replace(/\/$/, "");
  return `User-agent: *\nAllow: /\nSitemap: ${base}/sitemap.xml\n`;
}

function pageFile(outputDir, path) {
  const clean = path.replace(/^\/+|\/+$/g, "");
  return clean === "" ? join(outputDir, "index.html") : join(outputDir, clean, "index.html");
}

function write(file, contents) {
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, contents, "utf8");
}

/**
 * Emits `site` to `outputDir` as a deployable static tree; returns the sorted
 * list of written file paths.
 */
export function emitStaticSite(site, outputDir, configuration = SITE_CONFIGURATION) {
  const written = [];
  for (const page of site.pages) {
    const file = pageFile(outputDir, page.path);
    write(file, page.html);
    written.push(file);
  }
  const files = [
    [join(outputDir, "sitemap.xml"), site.sitemap],
    [join(outputDir, "robots.txt"), robotsTxt(configuration.origin)],
    [join(outputDir, "_headers"), HEADERS],
    [join(outputDir, "_redirects"), REDIRECTS],
  ];
  for (const [file, contents] of files) {
    write(file, contents);
    written.push(file);
  }
  // Copy static assets (favicon.svg, og.svg, ...) verbatim into the output.
  if (existsSync(ASSETS_DIR)) {
    mkdirSync(outputDir, { recursive: true });
    for (const name of readdirSync(ASSETS_DIR).sort()) {
      const destination = join(outputDir, name);
      copyFileSync(join(ASSETS_DIR, name), destination);
      written.push(destination);
    }
  }
  return written.sort();
}

/**
 * Compiles the full production site: the built-in inline pages (introduction,
 * getting-started, capabilities, CLI reference, concepts, gallery) combined with
 * every hand-written page under the repo `docs/` tree, loaded from disk and
 * published under `/docs/**`. Both sets flow through the same `compileSite`
 * pipeline, so navigation, search, and the sitemap cover all of them.
 */
export function buildFullSite(configuration = SITE_CONFIGURATION) {
  return compileSite([...docsContentPages(), ...loadDocsSources()], configuration);
}

/** Builds the production site and emits it to `outputDir`. */
export function renderStaticSite(outputDir) {
  return emitStaticSite(buildFullSite(SITE_CONFIGURATION), outputDir, SITE_CONFIGURATION);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const outputDir = process.argv[2] ?? "public";
  const written = renderStaticSite(outputDir);
  process.stdout.write(`Rendered ${written.length} file(s) to ${outputDir}\n`);
}
