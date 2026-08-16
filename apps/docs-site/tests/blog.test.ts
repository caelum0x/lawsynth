import { BlogCatalog } from "../src/blog.js";
import { parseMarkdown } from "../src/markdown.js";
import { deepEqual, equal, test, throws } from "./assertions.js";

const document = parseMarkdown("# Release notes\n\nExecutable documentation.");

test("blog catalog excludes drafts, sorts published posts, and indexes tags", () => {
  const catalog = new BlogCatalog([
    { slug: "first-release", title: "First release", description: "The first executable LawSynth release notes.", publishedAt: "2025-01-01T00:00:00Z", authors: ["LawSynth team"], tags: ["release"], document },
    { slug: "new-release", title: "New release", description: "A newer release with deterministic migration notes.", publishedAt: "2026-01-01T00:00:00Z", authors: ["LawSynth team"], tags: ["release", "migration"], document },
    { slug: "draft-post", title: "Draft", description: "A draft that must not appear in public archives.", publishedAt: "2026-02-01T00:00:00Z", authors: ["LawSynth team"], tags: [], document, draft: true },
  ]);
  deepEqual(catalog.posts.map((post) => post.slug), ["new-release", "first-release"]);
  equal(catalog.byTag("release").length, 2);
  deepEqual([...catalog.archive().keys()], ["2026", "2025"]);
});

test("blog catalog rejects duplicate slugs and missing authors", () => {
  throws(() => new BlogCatalog([{ slug: "same-post", title: "Same", description: "A valid description for the first post.", publishedAt: "2026-01-01T00:00:00Z", authors: ["Team"], tags: [], document }, { slug: "same-post", title: "Same again", description: "Another valid description for the second post.", publishedAt: "2026-01-02T00:00:00Z", authors: ["Team"], tags: [], document }]), /invalid blog slug/);
});
