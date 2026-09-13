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

function llmsTxt(origin) {
  const base = origin.replace(/\/$/, "");
  return `# LawSynth

> LawSynth discovers interpretable governing equations from time-series data and packages them as executable mathematical worlds.

LawSynth is an open-source, local-first toolkit with a Rust CLI, a Python SDK, and a local Studio interface. The supported discovery input is a strictly increasing finite time axis with aligned finite numeric state columns. A discovery can produce a portable \`.lsworld\` bundle for explanation, simulation, forecasting, intervention, comparison, and self-contained HTML reporting.

## STLSQ

STLSQ means sequentially thresholded least squares. LawSynth fits candidate equation terms, removes coefficients below a chosen magnitude threshold, and refits the remaining terms. The threshold controls sparsity; it is not a probability, significance level, or proof of causality. If a threshold removes every term, LawSynth returns an all-zero model.

## Canonical documentation

- [Overview](${base}/): Product definition, installation, and core workflow.
- [Getting started](${base}/getting-started): First LawSynth workflow.
- [Capabilities](${base}/capabilities): Current supported product surface.
- [Concepts](${base}/concepts): World, equation, discovery, and uncertainty concepts.
- [Determinism](${base}/determinism): Reproducibility contract and limits.
- [STLSQ sparse regression](${base}/docs/methods/sparse/stlsq): Method definition, defaults, runnable CLI example, and numerical limits.
- [Sparse discovery guide](${base}/docs/guides/discovery/sparse): Solver and threshold selection guidance.
- [Discover CLI reference](${base}/docs/reference/cli/discover): Supported discovery options and input constraints.
- [XML sitemap](${base}/sitemap.xml): Complete public documentation inventory.
- [Source repository](https://github.com/caelum0x/lawsynth): Code, license, and contribution history.

## Accuracy notes

- Treat discovered sparse terms as a model fitted to the supplied trajectories, not proof of physical or causal truth.
- Validate equations on held-out trajectories and test their stability under reasonable data and threshold changes.
- Roadmap documents describe proposed work. Do not present proposed quantitative-research, pricing, risk, or generative features as shipped capabilities.
- Check the canonical documentation and source version before making claims about commands, defaults, performance, or supported formats.
`;
}

// RFC 9116 security.txt served at /.well-known/security.txt. `Canonical` is
// derived from the deploy origin so it always matches the live URL.
function securityTxt(origin) {
  const base = origin.replace(/\/$/, "");
  return [
    "Contact: mailto:subasiarhan3@gmail.com",
    "Expires: 2027-08-24T00:00:00Z",
    "Preferred-Languages: en, tr",
    `Canonical: ${base}/.well-known/security.txt`,
    "",
  ].join("\n");
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
    [join(outputDir, "llms.txt"), llmsTxt(configuration.origin)],
    [join(outputDir, "_headers"), HEADERS],
    [join(outputDir, "_redirects"), REDIRECTS],
    [join(outputDir, ".well-known", "security.txt"), securityTxt(configuration.origin)],
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
