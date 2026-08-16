import { adjacentPages, buildNavigation } from "../src/navigation.js";
import { deepEqual, equal, test, throws } from "./assertions.js";

test("navigation groups visible pages and orders pages before computing adjacency", () => {
  const navigation = buildNavigation([
    { path: "/guide/configure", title: "Configure", section: "guide", order: 2 },
    { path: "/guide/install", title: "Install", section: "guide", order: 1 },
    { path: "/reference/api", title: "API", section: "reference" },
    { path: "/internal", title: "Internal", section: "guide", hidden: true },
  ], { guide: "Guides" });
  equal(navigation[0]!.title, "Guides");
  deepEqual(navigation[0]!.pages.map((page) => page.path), ["/guide/install", "/guide/configure"]);
  deepEqual(adjacentPages(navigation, "/guide/configure"), { previous: navigation[0]!.pages[0], next: navigation[1]!.pages[0] });
});

test("navigation rejects unsafe and duplicate paths", () => {
  throws(() => buildNavigation([{ path: "/guide/../secret", title: "Secret", section: "guide" }]), /invalid/);
  throws(() => buildNavigation([{ path: "/guide", title: "Guide", section: "guide" }, { path: "/guide", title: "Guide again", section: "guide" }]), /duplicate/);
});
