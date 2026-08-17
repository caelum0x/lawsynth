import type { Expression, WorldDefinition } from "@lawsynth/world-schema";
import { equationView, expressionSymbols, formatExpression, type EquationView } from "@lawsynth/world-viewer";
import type { ControlField, EquationBlock, EquationTerm, Metric, ScreenModel, ScreenSection } from "./types.js";

type Sign = "+" | "-";
function flip(sign: Sign): Sign {
  return sign === "+" ? "-" : "+";
}

/**
 * Flattens the top-level additive structure of an expression into signed terms.
 * `a - b*c` becomes `[+a, -b*c]`; `-(k*x)` becomes `[-k*x]`. Products, powers and
 * function calls are treated as atomic terms and pretty-printed whole.
 */
function collectTerms(expression: Expression, sign: Sign): readonly { readonly expression: Expression; readonly sign: Sign }[] {
  if (expression.kind === "binary" && expression.operator === "add") {
    return [...collectTerms(expression.left, sign), ...collectTerms(expression.right, sign)];
  }
  if (expression.kind === "binary" && expression.operator === "sub") {
    return [...collectTerms(expression.left, sign), ...collectTerms(expression.right, flip(sign))];
  }
  if (expression.kind === "unary" && expression.operator === "neg") {
    return collectTerms(expression.operand, flip(sign));
  }
  return [{ expression, sign }];
}

export function equationTerms(expression: Expression): readonly EquationTerm[] {
  return collectTerms(expression, "+").map((term, index) => ({
    id: `term-${index}`,
    sign: term.sign,
    text: formatExpression(term.expression),
    symbols: expressionSymbols(term.expression),
  }));
}

export interface EquationExplorerInput {
  readonly world: WorldDefinition;
  readonly selectedLawId?: string;
  readonly focusVariableId?: string;
}

function referencesVariable(view: EquationView, variableId: string): boolean {
  return view.target === variableId || view.symbols.includes(variableId);
}

export function equationExplorerModel(input: EquationExplorerInput): ScreenModel {
  const { world } = input;
  const timeSymbol = world.time.symbol ?? "t";
  const views = world.laws.map((law) => ({ law, view: equationView(law, timeSymbol) }));
  const selectedId = input.selectedLawId ?? views[0]?.view.id;

  const equations: readonly EquationBlock[] = views.map(({ law, view }) => {
    const selected = view.id === selectedId;
    return {
      id: view.id,
      heading: `${view.target ?? view.id} · ${view.kind}`,
      text: view.text,
      enabled: view.enabled,
      selected,
      terms: selected ? equationTerms(law.expression) : [],
    };
  });

  const lawOptions = views.map(({ view }) => ({ value: view.id, label: `${view.target ?? view.id} (${view.kind})` }));
  const variableOptions = [{ value: "", label: "All variables" }, ...world.variables.map((variable) => ({ value: variable.id, label: variable.name ?? variable.id }))];
  const focus = input.focusVariableId ?? "";
  const controls: readonly ControlField[] = [
    { id: "eq:law", label: "Law", kind: "select", value: selectedId ?? "", options: lawOptions },
    { id: "eq:variable", label: "Focus variable", kind: "select", value: focus, options: variableOptions, help: "Count and highlight laws that reference a variable." },
  ];

  const referencing = focus ? views.filter(({ view }) => referencesVariable(view, focus)).length : views.length;
  const selectedTermCount = equations.find((block) => block.selected)?.terms.length ?? 0;
  const metrics: readonly Metric[] = [
    { label: "Laws", value: String(views.length) },
    { label: "Disabled", value: String(views.filter(({ view }) => !view.enabled).length) },
    { label: focus ? `Refer to ${focus}` : "Variables", value: focus ? String(referencing) : String(world.variables.length) },
    { label: "Selected terms", value: String(selectedTermCount) },
  ];

  const sections: ScreenSection[] = [
    { kind: "metrics", id: "eq-metrics", title: "System", metrics },
    { kind: "controls", id: "eq-controls", title: "Selection", fields: controls },
    { kind: "equations", id: "eq-list", title: "Laws", equations },
  ];

  return { id: "equation-explorer", title: "Equation Explorer", subtitle: "Read discovered laws and inspect their terms", sections };
}
