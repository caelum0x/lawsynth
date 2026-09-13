import { compileSite } from "../src/site.js";
import { contains, deepEqual, equal, test, throws } from "./assertions.js";

const sources = [
  { path: "/guide/install", section: "guide", updatedAt: "2026-01-01T00:00:00Z", source: ["---", "title: Install", "description: Install the executable LawSynth native toolkit.", "order: 1", "tags: install", "---", "# Install", "", "Install the package."].join("\n") },
  { path: "/guide/discover", section: "guide", source: ["---", "title: Discover", "description: Discover sparse laws from aligned observations.", "order: 2", "tags: discovery", "---", "# Discover", "", "Run native discovery."].join("\n") },
  { path: "/internal", section: "guide", source: ["---", "title: Internal", "description: Internal documentation excluded from public output.", "draft: true", "---", "# Internal"].join("\n") },
] as const;

test("site compilation connects markdown, navigation, search, SEO, pagination, and sitemap", () => {
  const site = compileSite(sources, { origin: "https://docs.lawsynth.dev", name: "LawSynth", version: "0.1.0" });
  deepEqual(site.pages.map((page) => page.path), ["/guide/install", "/guide/discover"]);
  equal(site.search.search("discovery", { version: "0.1.0" })[0]!.document.path, "/guide/discover");
  contains(site.pages[0]!.html, 'rel="next" href="/guide/discover"');
  contains(site.pages[1]!.html, 'rel="prev" href="/guide/install"');
  contains(site.pages[0]!.html, "article:modified_time");
  contains(site.sitemap, "https://docs.lawsynth.dev/guide/install");
});

test("site compilation caps long metadata descriptions without cutting a word", () => {
  const longDescription = `${"A useful description with several complete words. ".repeat(6)}Final sentence.`;
  const site = compileSite(
    [{ path: "/long", section: "guide", source: ["---", "title: Long page", `description: ${longDescription}`, "---", "# Long page"].join("\n") }],
    { origin: "https://docs.lawsynth.dev" },
  );
  const match = site.pages[0]!.html.match(/<meta name="description" content="([^"]+)">/);
  equal(match?.[1]?.length !== undefined && match[1].length <= 155, true);
  equal(match?.[1]?.endsWith("…"), true);
});

test("site compilation can include drafts but rejects insecure origins and duplicate pages", () => {
  equal(compileSite(sources, { origin: "https://docs.lawsynth.dev", includeDrafts: true }).pages.length, 3);
  throws(() => compileSite(sources, { origin: "http://docs.lawsynth.dev" }), /HTTPS/);
  throws(() => compileSite([sources[0], { ...sources[0], source: "# Different" }], { origin: "https://docs.lawsynth.dev" }), /duplicate/);
});
