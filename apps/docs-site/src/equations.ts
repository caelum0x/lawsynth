import type { Expression, Law, WorldDefinition } from "@lawsynth/world-schema";
import { equationView, formatExpression } from "@lawsynth/world-viewer";
import { escapeHtml } from "./code.js";
export interface EquationReference { readonly id: string; readonly target?: string; readonly plainText: string; readonly html: string; readonly description?: string; }
function equationHtml(text: string): string { return `<code class="equation" aria-label="${escapeHtml(text)}">${escapeHtml(text)}</code>`; }
export function renderEquation(expression: Expression): string { return equationHtml(formatExpression(expression)); }
export function equationReference(law: Law, timeSymbol = "t"): EquationReference {
  const view = equationView(law, timeSymbol);
  return Object.freeze({ id: view.id, ...(view.target === undefined ? {} : { target: view.target }), plainText: view.text, html: equationHtml(view.text), ...(view.description === undefined ? {} : { description: view.description }) });
}
export function worldEquationReferences(world: WorldDefinition): readonly EquationReference[] {
  return Object.freeze(world.laws.map((law) => equationReference(law, world.time.symbol ?? "t")));
}
