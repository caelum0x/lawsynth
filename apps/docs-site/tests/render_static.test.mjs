// Plain-ESM test for the static-site emitter (run directly by `node --test`,
// not compiled by tsc — it uses Node's fs/os which the pure-TS build excludes).

import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { buildDocsSite } from "../dist/src/content.js";
import { emitStaticSite, renderStaticSite, SITE_CONFIGURATION } from "../render-static.mjs";

test("emitStaticSite writes one index.html per page plus control files", () => {
  const dir = mkdtempSync(join(tmpdir(), "lawsynth-site-"));
  try {
    const site = buildDocsSite(SITE_CONFIGURATION);
    const written = emitStaticSite(site, dir);

    assert.ok(written.includes(join(dir, "index.html")), "landing page at the root");
    for (const page of site.pages) {
      const file =
        page.path === "/"
          ? join(dir, "index.html")
          : join(dir, page.path.replace(/^\//, ""), "index.html");
      assert.ok(written.includes(file), `page ${page.path} was emitted`);
      assert.ok(readFileSync(file, "utf8").startsWith("<!doctype html>"), `${page.path} is a full HTML document`);
    }

    for (const name of ["sitemap.xml", "robots.txt", "_headers", "_redirects"]) {
      assert.ok(written.includes(join(dir, name)), `${name} was written`);
    }
    assert.ok(readFileSync(join(dir, "sitemap.xml"), "utf8").includes("https://lawsynth.dev"), "sitemap origin");
    assert.ok(
      readFileSync(join(dir, "robots.txt"), "utf8").includes("Sitemap: https://lawsynth.dev/sitemap.xml"),
      "robots points to the sitemap",
    );
    assert.ok(readFileSync(join(dir, "_headers"), "utf8").includes("X-Content-Type-Options: nosniff"), "security headers");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("the emitted tree is deterministic (byte-identical across two renders)", () => {
  const a = mkdtempSync(join(tmpdir(), "lawsynth-site-a-"));
  const b = mkdtempSync(join(tmpdir(), "lawsynth-site-b-"));
  try {
    const filesA = renderStaticSite(a);
    const filesB = renderStaticSite(b);
    assert.equal(filesA.length, filesB.length);
    for (let i = 0; i < filesA.length; i += 1) {
      assert.equal(filesA[i].slice(a.length), filesB[i].slice(b.length), "same relative paths in order");
      assert.equal(readFileSync(filesA[i], "utf8"), readFileSync(filesB[i], "utf8"), "byte-identical content");
    }
  } finally {
    rmSync(a, { recursive: true, force: true });
    rmSync(b, { recursive: true, force: true });
  }
});
