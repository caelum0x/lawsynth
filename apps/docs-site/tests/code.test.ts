import { codeBlock, escapeHtml, highlightCode, normalizeLanguage } from "../src/code.js";
import { contains, equal, test } from "./assertions.js";

test("code blocks normalize aliases and escape untrusted source before highlighting", () => {
  const block = codeBlock('const html = "<tag>";', "js", "SDK setup");
  equal(block.language, "typescript");
  equal(block.caption, "SDK setup");
  contains(block.highlightedHtml, '<span class="tok-keyword">const</span>');
  contains(block.highlightedHtml, "&lt;tag&gt;");
  equal(escapeHtml("'\"&<>"), "&#39;&quot;&amp;&lt;&gt;");
});

test("code presentation falls back to text for unsupported fences", () => {
  equal(normalizeLanguage("fortran"), "text");
  contains(highlightCode("$ lawsynth discover", "bash"), 'class="tok-prompt"');
});
