// Docs loader for the static-site emitter (lawsynth.dev).
//
// The documentation SSG (`./dist/src/*.js`) is deliberately Node-free: it turns
// an array of in-memory `DocumentationPageSource` objects into a compiled site.
// The repository's real, hand-written documentation lives as ~250 Markdown files
// under the repo-root `docs/` tree. This module is the bridge: it reads those
// files from disk (fs is allowed here, in a `.mjs`, unlike `src/content.ts`) and
// maps each one to a `DocumentationPageSource` the SSG can compile.
//
// It runs only at render time (`render-static.mjs`), so the pure TypeScript
// build and its tests never depend on the filesystem. Output is fully
// deterministic: files are discovered in sorted order and no timestamps are read.

import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

/** Repo-root `docs/` directory, resolved relative to this script. */
const DOCS_DIR = fileURLToPath(new URL("../../docs", import.meta.url));

/** Site prefix under which every docs page is published. */
const DOCS_PREFIX = "/docs";

/**
 * Recursively collect every `*.md` file under `dir`, returned as POSIX-style
 * paths relative to the docs root, sorted for deterministic output.
 */
function collectMarkdownFiles(dir) {
  const files = [];
  const walk = (absolute, relative) => {
    for (const entry of readdirSync(absolute, { withFileTypes: true })) {
      const childAbsolute = path.join(absolute, entry.name);
      const childRelative = relative ? `${relative}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        walk(childAbsolute, childRelative);
      } else if (entry.isFile() && entry.name.endsWith(".md")) {
        files.push(childRelative);
      }
    }
  };
  walk(dir, "");
  return files.sort();
}

/**
 * Map a docs-relative path (e.g. `guide/README.md`) to its stable site path.
 * `README.md` becomes the directory index, everything else drops `.md`.
 *   guide/README.md      -> /docs/guide
 *   guide/workflow.md    -> /docs/guide/workflow
 *   README.md            -> /docs
 */
function toSitePath(relative) {
  if (relative === "README.md") return DOCS_PREFIX;
  if (relative.endsWith("/README.md")) {
    return `${DOCS_PREFIX}/${relative.slice(0, -"/README.md".length)}`;
  }
  return `${DOCS_PREFIX}/${relative.slice(0, -".md".length)}`;
}

/** The top-level folder under docs/ that drives navigation grouping. */
function sectionFor(relative) {
  const slash = relative.indexOf("/");
  return slash < 0 ? "docs" : relative.slice(0, slash);
}

/** True for links that already carry a URL scheme (http:, https:, mailto:, ...). */
function hasScheme(target) {
  return /^[a-z][a-z0-9+.-]*:/iu.test(target) || target.startsWith("//");
}

/**
 * Rewrite a single link target found inside `fromRelative`'s Markdown. Relative
 * `.md` links that resolve inside the docs tree become `/docs/...` site paths
 * (with the `.md` dropped and any `#fragment` preserved); everything else —
 * external URLs, absolute paths, anchors, non-`.md` files, and `.md` links that
 * escape the docs tree — is returned unchanged.
 */
function rewriteTarget(target, fromRelative) {
  const trimmed = target.trim();
  if (trimmed === "" || trimmed.startsWith("#") || trimmed.startsWith("/")) {
    return target;
  }
  if (hasScheme(trimmed)) return target;

  const hash = trimmed.indexOf("#");
  const base = hash < 0 ? trimmed : trimmed.slice(0, hash);
  const fragment = hash < 0 ? "" : trimmed.slice(hash);
  if (!base.toLowerCase().endsWith(".md")) return target;

  const fromDir = path.posix.dirname(fromRelative);
  const resolved = path.posix.normalize(path.posix.join(fromDir, base));
  if (resolved.startsWith("..") || resolved.startsWith("/")) {
    // Escapes the docs tree (e.g. ../../specs/README.md): not a published page.
    return target;
  }
  return toSitePath(resolved) + fragment;
}

const LINK = /\[([^\]]+)\]\(([^)\s]+)\)/gu;

/**
 * Rewrite every internal `.md` link in `source`, skipping fenced code blocks so
 * example code is emitted verbatim.
 */
function rewriteLinks(source, fromRelative) {
  let inFence = false;
  return source
    .split("\n")
    .map((line) => {
      if (line.startsWith("```")) {
        inFence = !inFence;
        return line;
      }
      if (inFence) return line;
      return line.replace(
        LINK,
        (match, label, target) =>
          `[${label}](${rewriteTarget(target, fromRelative)})`,
      );
    })
    .join("\n");
}

/** Title-case a section id the same way the navigation builder does. */
function sectionTitle(id) {
  return id
    .replaceAll("-", " ")
    .replace(/\b\w/gu, (letter) => letter.toUpperCase());
}

/**
 * A `/docs` landing page linking to each section's entry page, so the whole
 * docs corpus has a single discoverable home (also used by the footer/nav).
 * Built purely from the already-sorted sources, so it stays deterministic.
 */
function landingSource(sources) {
  const seen = new Map();
  for (const source of sources) {
    const section = sectionFor(pathToRelative(source.path));
    if (!seen.has(section)) seen.set(section, source.path);
    // Prefer the section's own index page when it exists.
    if (source.path === `${DOCS_PREFIX}/${section}`) seen.set(section, source.path);
  }
  const items = [...seen.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([section, href]) => `- [${sectionTitle(section)}](${href})`);
  return [
    "# LawSynth documentation",
    "",
    "The complete LawSynth documentation, generated from the project's `docs/` tree — concepts, guides, methods, reference, self-hosting, research, and more. Use the navigation to browse every page, or jump into a section:",
    "",
    ...items,
    "",
  ].join("\n");
}

/** Invert `toSitePath` enough to recover the section for the landing builder. */
function pathToRelative(sitePath) {
  return sitePath.slice(DOCS_PREFIX.length + 1);
}

/**
 * Load every docs Markdown file as a `DocumentationPageSource`: a `/docs/...`
 * path, a `section` = its top-level folder, and its Markdown with internal `.md`
 * links rewritten to site paths. A `/docs` landing page is prepended so the
 * corpus has a single home. The result is deterministic and fs-time-free.
 */
export function loadDocsSources() {
  const files = collectMarkdownFiles(DOCS_DIR);
  const pageSources = files.map((relative) => {
    const raw = readFileSync(path.join(DOCS_DIR, relative), "utf8");
    return Object.freeze({
      path: toSitePath(relative),
      section: sectionFor(relative),
      source: rewriteLinks(raw, relative),
    });
  });

  // Only synthesize a landing if no real page already claims `/docs`.
  if (pageSources.some((source) => source.path === DOCS_PREFIX)) {
    return Object.freeze(pageSources);
  }

  const landing = Object.freeze({
    path: DOCS_PREFIX,
    section: "docs",
    source: landingSource(pageSources),
  });

  return Object.freeze([landing, ...pageSources]);
}
