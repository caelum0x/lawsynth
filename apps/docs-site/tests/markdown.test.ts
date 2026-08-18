import { parseMarkdown, renderMarkdown } from "../src/markdown.js";
import { contains, deepEqual, equal, test, throws } from "./assertions.js";

test("markdown parsing creates structured blocks, metadata, stable anchors, and searchable text", () => {
  const fence = String.fromCharCode(96).repeat(3);
  const document = parseMarkdown([
    "---", "title: Discovery guide", "description: Learn how to run deterministic sparse discovery.", "order: 2", "tags: discovery, python", "draft: false", "---",
    "# Getting started", "", "A <safe> paragraph with **emphasis** and [guide](/guide).", "", "## Getting started", "",
    "- Install the wheel", "- Load observations", "", "> Native execution only.", "", fence + "python", "from lawsynth import discover", fence, "", "---",
  ].join("\n"));
  deepEqual(document.metadata, { title: "Discovery guide", description: "Learn how to run deterministic sparse discovery.", order: 2, draft: false, tags: ["discovery", "python"] });
  deepEqual(document.headings.map((heading) => heading.id), ["getting-started", "getting-started-2"]);
  contains(document.plainText, "Native execution only.");
  const html = renderMarkdown(document);
  contains(html, "&lt;safe&gt;");
  contains(html, '<a href="/guide">guide</a>');
  contains(html, 'data-language="python"');
});

test("standalone image lines become figures, with alt text indexed and unsafe sources rejected", () => {
  const document = parseMarkdown([
    "# Diagram", "", "![The discovery loop](/loop.svg)", "", "![](/hero.svg)", "", "![tracker](https://evil.example/pixel.gif)", "", "![data uri](data:image/svg+xml;base64,AAAA)",
  ].join("\n"));
  const imageBlocks = document.blocks.filter((block) => block.kind === "image");
  equal(imageBlocks.length, 2);
  contains(document.plainText, "The discovery loop");
  const html = renderMarkdown(document);
  contains(html, '<figure class="docs-figure"><img src="/loop.svg" alt="The discovery loop" loading="lazy" decoding="async"><figcaption>The discovery loop</figcaption></figure>');
  contains(html, '<img src="/hero.svg" alt="" loading="lazy" decoding="async">');
  // The off-origin and data: sources are never rendered as an <img> — only the
  // two same-origin figures above are. They fall through to the paragraph path.
  equal((html.match(/<img /gu) ?? []).length, 2);
  equal(/<img[^>]+evil\.example/u.test(html), false);
  equal(/<img[^>]+data:/u.test(html), false);
});

test("markdown rejects unsupported or malformed front matter and fences", () => {
  throws(() => parseMarkdown(["---", "author: someone", "---", "# Title"].join("\n")), /unsupported/);
  throws(() => parseMarkdown([String.fromCharCode(96).repeat(3) + "python", "print('missing close')"].join("\n")), /unterminated/);
  equal(parseMarkdown("# Only heading").plainText, "Only heading");
});
