import type { WorldDefinition } from "@lawsynth/world-schema";
import { equationsForWorld } from "@lawsynth/world-viewer";
import { worldToLatex, worldToPython } from "./export-format.js";
import type { CodeBlock, EquationBlock, Metric, ScreenModel, ScreenSection } from "./types.js";

export interface ExportScreenInput {
  readonly world: WorldDefinition;
}

/**
 * The in-app "take your model" surface. It previews exactly what the world
 * contains — rendered equations, a LaTeX system, a standalone Python module,
 * and the raw World IR — each with a copy-to-clipboard affordance wired in the
 * render description. Nothing is fabricated; every artifact is derived from the
 * validated `WorldDefinition`.
 */
export function exportScreenModel(input: ExportScreenInput): ScreenModel {
  const { world } = input;
  const views = equationsForWorld(world);

  const metrics: readonly Metric[] = [
    { label: "Laws", value: String(world.laws.length) },
    { label: "Variables", value: String(world.variables.length) },
    { label: "Parameters", value: String(world.parameters?.length ?? 0) },
    { label: "Format", value: world.formatVersion },
  ];

  const equations: readonly EquationBlock[] = views.map((view) => ({
    id: view.id,
    heading: `${view.target ?? view.id} · ${view.kind}`,
    text: view.text,
    enabled: view.enabled,
    selected: false,
    terms: [],
  }));

  const blocks: readonly CodeBlock[] = [
    { id: "latex", label: "LaTeX", language: "latex", content: worldToLatex(world), caption: "Aligned system for papers and reports." },
    { id: "python", label: "Python", language: "python", content: worldToPython(world), caption: "Runnable module: parameters, derivatives(), and algebraic laws." },
    { id: "json", label: "World IR", language: "json", content: JSON.stringify(world, null, 2), caption: "The validated .lsworld payload — the source every surface shares." },
  ];

  const sections: readonly ScreenSection[] = [
    { kind: "metrics", id: "export-metrics", title: "Model", metrics },
    { kind: "equations", id: "export-equations", title: "Rendered equations", equations },
    { kind: "code", id: "export-code", title: "Export", blocks },
  ];

  return {
    id: "export-screen",
    title: "Export",
    subtitle: "Take your model: equations, LaTeX, Python, and the raw World IR",
    sections,
  };
}
