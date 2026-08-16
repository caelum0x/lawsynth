import type { ViewerExpression, ViewerLaw } from "./viewer.js";

export interface EquationModel { readonly id: string; readonly target?: string; readonly kind: string; readonly text: string; readonly enabled: boolean; readonly symbols: readonly string[]; }
const operation: Readonly<Record<string, string>> = { add: "+", sub: "−", mul: "×", div: "÷", pow: "^", eq: "=", ne: "≠", lt: "<", lte: "≤", gt: ">", gte: "≥", and: "∧", or: "∨" };

/** Formats the typed expression AST, never evaluates arbitrary source text. */
export function formatExpression(expression: ViewerExpression): string {
  const node = expression as Record<string, unknown>;
  const expressionAt = (key: string): ViewerExpression => node[key] as ViewerExpression;
  switch (node.kind) {
    case "constant": return String(node.value);
    case "symbol": return String(node.id);
    case "unary": return `${node.operator === "neg" ? "−" : `${String(node.operator)}(`}${formatExpression(expressionAt("operand"))}${node.operator === "neg" ? "" : ")"}`;
    case "not": return `¬(${formatExpression(expressionAt("operand"))})`;
    case "binary": case "comparison": return `(${formatExpression(expressionAt("left"))} ${operation[String(node.operator)] ?? String(node.operator)} ${formatExpression(expressionAt("right"))})`;
    case "logical": return `(${((node.operands as readonly ViewerExpression[] | undefined) ?? []).map(formatExpression).join(` ${operation[String(node.operator)] ?? String(node.operator)} `)})`;
    case "delay": return `delay(${formatExpression(expressionAt("expression"))}, ${String(node.lag)})`;
    case "call": return `${String(node.function)}(${((node.arguments as readonly ViewerExpression[] | undefined) ?? []).map(formatExpression).join(", ")})`;
    case "piecewise": return `piecewise(${((node.branches as readonly Record<string, ViewerExpression>[] | undefined) ?? []).map((branch) => `${formatExpression(branch.then)} if ${formatExpression(branch.when)}`).join("; ")}; otherwise ${formatExpression(expressionAt("otherwise"))})`;
    default: throw new TypeError(`unknown expression kind ${String(node.kind)}`);
  }
}

export function expressionSymbols(expression: ViewerExpression): readonly string[] {
  const symbols = new Set<string>(); const walk = (node: unknown): void => { if (!node || typeof node !== "object") return; const value = node as Record<string, unknown>; if (value.kind === "symbol" && typeof value.id === "string") symbols.add(value.id); for (const child of Object.values(value)) if (child && typeof child === "object") Array.isArray(child) ? child.forEach(walk) : walk(child); }; walk(expression); return [...symbols].sort();
}
export function buildEquationModel(law: ViewerLaw): EquationModel { return { id: law.id, ...(law.target === undefined ? {} : { target: law.target }), kind: law.kind, text: formatExpression(law.expression), enabled: law.enabled !== false, symbols: expressionSymbols(law.expression) }; }
import type { Expression, Law, WorldDefinition } from "@lawsynth/world-schema";

export interface EquationView {
  readonly id: string;
  readonly target?: string;
  readonly kind: Law["kind"];
  readonly text: string;
  readonly symbols: readonly string[];
  readonly description?: string;
  readonly enabled: boolean;
}

const binarySymbols: Readonly<Record<string, string>> = {
  add: "+", sub: "−", mul: "×", div: "/", pow: "^", min: "min", max: "max",
};
const comparisonSymbols: Readonly<Record<string, string>> = {
  eq: "=", ne: "≠", lt: "<", lte: "≤", gt: ">", gte: "≥",
};

function precedence(expression: Expression): number {
  if (expression.kind !== "binary") return 100;
  if (expression.operator === "add" || expression.operator === "sub") return 10;
  if (expression.operator === "mul" || expression.operator === "div") return 20;
  return 30;
}

function childText(expression: Expression, parentPrecedence: number): string {
  const rendered = formatExpression(expression);
  return precedence(expression) < parentPrecedence ? `(${rendered})` : rendered;
}

/** Produces deterministic plain-text mathematics suitable for textContent and screen readers. */
export function formatExpression(expression: Expression): string {
  switch (expression.kind) {
    case "constant":
      return typeof expression.value === "string" ? JSON.stringify(expression.value) : String(expression.value);
    case "symbol": return expression.id;
    case "unary":
      return expression.operator === "neg"
        ? `−${childText(expression.operand, 40)}`
        : `${expression.operator}(${formatExpression(expression.operand)})`;
    case "binary": {
      if (expression.operator === "min" || expression.operator === "max") {
        return `${expression.operator}(${formatExpression(expression.left)}, ${formatExpression(expression.right)})`;
      }
      const rank = precedence(expression);
      return `${childText(expression.left, rank)} ${binarySymbols[expression.operator]} ${childText(expression.right, rank + (expression.operator === "pow" ? -1 : 0))}`;
    }
    case "comparison":
      return `${formatExpression(expression.left)} ${comparisonSymbols[expression.operator]} ${formatExpression(expression.right)}`;
    case "logical": return expression.operands.map(formatExpression).join(expression.operator === "and" ? " ∧ " : " ∨ ");
    case "not": return `¬(${formatExpression(expression.operand)})`;
    case "delay": return `${formatExpression(expression.expression)}[t − ${expression.lag}]`;
    case "piecewise": {
      const branches = expression.branches.map((branch) => `${formatExpression(branch.then)} if ${formatExpression(branch.when)}`);
      return `{ ${[...branches, `${formatExpression(expression.otherwise)} otherwise`].join("; ")} }`;
    }
    case "call": return `${expression.function}(${expression.arguments.map(formatExpression).join(", ")})`;
  }
}

export function expressionSymbols(expression: Expression): readonly string[] {
  const values = new Set<string>();
  const stack: Expression[] = [expression];
  while (stack.length > 0) {
    const current = stack.pop();
    if (current === undefined) continue;
    switch (current.kind) {
      case "symbol": values.add(current.id); break;
      case "unary": case "not": stack.push(current.operand); break;
      case "binary": case "comparison": stack.push(current.right, current.left); break;
      case "logical": stack.push(...current.operands); break;
      case "delay": stack.push(current.expression); break;
      case "piecewise":
        stack.push(current.otherwise, ...current.branches.flatMap((branch) => [branch.when, branch.then]));
        break;
      case "call": stack.push(...current.arguments); break;
      case "constant": break;
    }
  }
  return [...values].sort((left, right) => left.localeCompare(right));
}

export function equationView(law: Law, timeSymbol = "t"): EquationView {
  const target = "target" in law ? law.target : undefined;
  let left = law.id;
  if (target !== undefined) {
    left = law.kind === "continuous" ? `d${target}/d${timeSymbol}` : law.kind === "discrete" ? `${target}[${timeSymbol} + 1]` : target;
  }
  const expression = `${left} = ${formatExpression(law.expression)}`;
  return Object.freeze({
    id: law.id,
    ...(target === undefined ? {} : { target }),
    kind: law.kind,
    text: expression,
    symbols: Object.freeze([...expressionSymbols(law.expression)]),
    ...(law.description === undefined ? {} : { description: law.description }),
    enabled: law.enabled !== false,
  });
}

export function equationsForWorld(world: WorldDefinition): readonly EquationView[] {
  return Object.freeze(world.laws.map((law) => equationView(law, world.time.symbol ?? "t")));
}
