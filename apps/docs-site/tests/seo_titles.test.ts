import { fallbackDescription, uniqueSeoTitles } from "../src/seo_titles.js";
import { equal, test } from "./assertions.js";

test("unique SEO titles keep the primary page plain and qualify the duplicates", () => {
  const titles = uniqueSeoTitles([
    { path: "/getting-started", title: "Getting started" },
    { path: "/docs/getting-started", title: "Getting started" },
    { path: "/docs/guide/getting-started", title: "Getting started" },
    { path: "/systems/lorenz", title: "Lorenz system" },
    { path: "/examples/lorenz", title: "Lorenz system" },
    { path: "/docs/guides/simulation/ensembles", title: "Ensembles" },
    { path: "/docs/concepts/uncertainty/ensembles", title: "Ensembles" },
    { path: "/capabilities", title: "Capabilities" },
  ]);
  equal(titles.get("/getting-started"), "Getting started");
  equal(titles.get("/docs/getting-started"), "Getting started (Docs)");
  equal(titles.get("/docs/guide/getting-started"), "Getting started (Guide)");
  equal(titles.get("/systems/lorenz"), "Lorenz system");
  equal(titles.get("/examples/lorenz"), "Lorenz system (Examples)");
  equal(titles.get("/docs/concepts/uncertainty/ensembles"), "Ensembles");
  equal(titles.get("/docs/guides/simulation/ensembles"), "Ensembles (Guides / Simulation)");
  equal(titles.get("/capabilities"), "Capabilities");
  equal(new Set(titles.values()).size, titles.size);
});

test("fallback description drops the repeated heading and markdown markers", () => {
  equal(
    fallbackDescription("Core concepts", "Core concepts A **Dataset** is a validated `numeric` time axis."),
    "A Dataset is a validated numeric time axis.",
  );
  equal(fallbackDescription("Other", "Plain text body."), "Plain text body.");
});
