import { SearchIndex } from "../src/search.js";
import { contains, deepEqual, equal, test, throws } from "./assertions.js";

test("search index ranks weighted document fields and filters versions", () => {
  const index = new SearchIndex();
  index.add({ id: "guide", path: "/guide/discovery", title: "Sparse discovery", headings: ["Native discovery"], tags: ["discovery"], text: "Run sparse regression against observed trajectories.", version: "0.1.0" });
  index.add({ id: "blog", path: "/blog/release", title: "Release notes", text: "Discovery updates and migration notes.", version: "0.2.0" });
  const hits = index.search("the discovery", { version: "0.1.0" });
  deepEqual(hits.map((hit) => hit.document.id), ["guide"]);
  equal(hits[0]!.terms.join(","), "discovery");
  contains(hits[0]!.excerpt, "sparse regression");
});

test("search handles no useful terms and rejects invalid limits or duplicate ids", () => {
  const index = new SearchIndex();
  index.add({ id: "one", path: "/one", title: "One", text: "A documented page." });
  deepEqual(index.search("the and"), []);
  throws(() => index.search("page", { limit: 0 }), /1..100/);
  throws(() => index.add({ id: "one", path: "/two", title: "Two", text: "Another page." }), /duplicate/);
});
