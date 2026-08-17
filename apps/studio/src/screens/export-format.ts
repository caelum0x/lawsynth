import type { Expression, WorldDefinition } from "@lawsynth/world-schema";

/**
 * A dependency-free expression renderer that turns the validated World IR
 * `Expression` union into LaTeX and standalone Python. It mirrors the CLI
 * `export` surface: no fabrication, every operator maps to exactly what the
 * expression tree contains. Precedence is tracked so parentheses are added
 * only where the target syntax requires them.
 */

type Rank = number;
const ADD: Rank = 10;
const MUL: Rank = 20;
const POW: Rank = 30;
const ATOM: Rank = 100;

function rankOf(expression: Expression): Rank {
  if (expression.kind !== "binary") return ATOM;
  if (expression.operator === "add" || expression.operator === "sub") return ADD;
  if (expression.operator === "mul" || expression.operator === "div") return MUL;
  if (expression.operator === "pow") return POW;
  return ATOM;
}

const UNARY_FN: Readonly<Record<string, string>> = { abs: "abs", exp: "exp", log: "log", sqrt: "sqrt", sin: "sin", cos: "cos", tan: "tan" };
const COMPARISON: Readonly<Record<string, { latex: string; python: string }>> = {
  eq: { latex: "=", python: "==" },
  ne: { latex: "\\neq", python: "!=" },
  lt: { latex: "<", python: "<" },
  lte: { latex: "\\leq", python: "<=" },
  gt: { latex: ">", python: ">" },
  gte: { latex: "\\geq", python: ">=" },
};

function constant(value: number | boolean | string, target: "latex" | "python"): string {
  if (typeof value === "boolean") return target === "python" ? (value ? "True" : "False") : value ? "\\text{true}" : "\\text{false}";
  if (typeof value === "string") return JSON.stringify(value);
  return String(value);
}

function escapeLatexSymbol(id: string): string {
  return id.replace(/_/gu, "\\_");
}

// ---------------------------------------------------------------------------
// LaTeX
// ---------------------------------------------------------------------------

function latexChild(expression: Expression, parentRank: Rank): string {
  const text = expressionToLatex(expression);
  return rankOf(expression) < parentRank ? `\\left(${text}\\right)` : text;
}

export function expressionToLatex(expression: Expression): string {
  switch (expression.kind) {
    case "constant":
      return constant(expression.value, "latex");
    case "symbol":
      return escapeLatexSymbol(expression.id);
    case "unary": {
      if (expression.operator === "neg") return `-${latexChild(expression.operand, POW)}`;
      const inner = expressionToLatex(expression.operand);
      if (expression.operator === "sqrt") return `\\sqrt{${inner}}`;
      if (expression.operator === "exp") return `e^{${inner}}`;
      if (expression.operator === "log") return `\\ln\\left(${inner}\\right)`;
      if (expression.operator === "abs") return `\\left|${inner}\\right|`;
      return `\\${expression.operator}\\left(${inner}\\right)`;
    }
    case "binary": {
      const { operator } = expression;
      if (operator === "div") return `\\frac{${expressionToLatex(expression.left)}}{${expressionToLatex(expression.right)}}`;
      if (operator === "pow") return `${latexChild(expression.left, ATOM)}^{${expressionToLatex(expression.right)}}`;
      if (operator === "min" || operator === "max") return `\\${operator}\\left(${expressionToLatex(expression.left)}, ${expressionToLatex(expression.right)}\\right)`;
      const rank = rankOf(expression);
      const symbol = operator === "add" ? "+" : operator === "sub" ? "-" : "\\cdot";
      return `${latexChild(expression.left, rank)} ${symbol} ${latexChild(expression.right, rank)}`;
    }
    case "comparison":
      return `${expressionToLatex(expression.left)} ${COMPARISON[expression.operator]!.latex} ${expressionToLatex(expression.right)}`;
    case "logical":
      return expression.operands.map((operand) => latexChild(operand, ADD)).join(expression.operator === "and" ? " \\land " : " \\lor ");
    case "not":
      return `\\neg\\left(${expressionToLatex(expression.operand)}\\right)`;
    case "delay":
      return `${latexChild(expression.expression, ATOM)}\\left[t - ${expression.lag}\\right]`;
    case "piecewise": {
      const rows = [
        ...expression.branches.map((branch) => `${expressionToLatex(branch.then)} & \\text{if } ${expressionToLatex(branch.when)}`),
        `${expressionToLatex(expression.otherwise)} & \\text{otherwise}`,
      ];
      return `\\begin{cases} ${rows.join(" \\\\ ")} \\end{cases}`;
    }
    case "call":
      return `\\operatorname{${escapeLatexSymbol(expression.function)}}\\left(${expression.arguments.map(expressionToLatex).join(", ")}\\right)`;
  }
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

function pythonChild(expression: Expression, parentRank: Rank): string {
  const text = expressionToPython(expression);
  return rankOf(expression) < parentRank ? `(${text})` : text;
}

export function expressionToPython(expression: Expression): string {
  switch (expression.kind) {
    case "constant":
      return constant(expression.value, "python");
    case "symbol":
      return expression.id;
    case "unary": {
      if (expression.operator === "neg") return `-${pythonChild(expression.operand, POW)}`;
      const inner = expressionToPython(expression.operand);
      if (expression.operator === "abs") return `abs(${inner})`;
      return `math.${UNARY_FN[expression.operator]}(${inner})`;
    }
    case "binary": {
      const { operator } = expression;
      if (operator === "min" || operator === "max") return `${operator}(${expressionToPython(expression.left)}, ${expressionToPython(expression.right)})`;
      if (operator === "pow") return `${pythonChild(expression.left, ATOM)} ** ${pythonChild(expression.right, POW)}`;
      const rank = rankOf(expression);
      const symbol = operator === "add" ? "+" : operator === "sub" ? "-" : operator === "mul" ? "*" : "/";
      // Right operand of subtraction/division needs the next precedence tier to keep grouping.
      const rightRank = operator === "sub" || operator === "div" ? rank + 1 : rank;
      return `${pythonChild(expression.left, rank)} ${symbol} ${pythonChild(expression.right, rightRank)}`;
    }
    case "comparison":
      return `${expressionToPython(expression.left)} ${COMPARISON[expression.operator]!.python} ${expressionToPython(expression.right)}`;
    case "logical":
      return expression.operands.map((operand) => pythonChild(operand, ADD)).join(expression.operator === "and" ? " and " : " or ");
    case "not":
      return `not (${expressionToPython(expression.operand)})`;
    case "delay":
      return `delay(${expressionToPython(expression.expression)}, ${expression.lag})`;
    case "piecewise": {
      // Fold branches into nested conditional expressions, most specific first.
      return [...expression.branches].reverse().reduce(
        (otherwise, branch) => `(${expressionToPython(branch.then)} if ${expressionToPython(branch.when)} else ${otherwise})`,
        expressionToPython(expression.otherwise),
      );
    }
    case "call":
      return `${expression.function}(${expression.arguments.map(expressionToPython).join(", ")})`;
  }
}

// ---------------------------------------------------------------------------
// World-level documents
// ---------------------------------------------------------------------------

function lawLeftLatex(target: string, kind: string, timeSymbol: string): string {
  if (kind === "continuous") return `\\frac{d${escapeLatexSymbol(target)}}{d${escapeLatexSymbol(timeSymbol)}}`;
  if (kind === "discrete") return `${escapeLatexSymbol(target)}_{${escapeLatexSymbol(timeSymbol)}+1}`;
  return escapeLatexSymbol(target);
}

/** Renders every law of a world as an aligned LaTeX system (align* environment). */
export function worldToLatex(world: WorldDefinition): string {
  const timeSymbol = world.time.symbol ?? "t";
  const lines = world.laws.map((law) => {
    const target = "target" in law ? law.target : law.id;
    const left = "target" in law ? lawLeftLatex(target, law.kind, timeSymbol) : escapeLatexSymbol(law.id);
    return `  ${left} &= ${expressionToLatex(law.expression)}`;
  });
  return `\\begin{align*}\n${lines.join(" \\\\\n")}\n\\end{align*}`;
}

/**
 * Emits a runnable, dependency-light Python module mirroring the CLI export:
 * parameters as a dict, a `derivatives` function evaluating each continuous law,
 * and discrete/algebraic laws surfaced as an `algebraic` map. Parameters and
 * state are unpacked into locals so each law reads exactly as authored — only
 * what the world declares is written, no synthetic values.
 */
export function worldToPython(world: WorldDefinition): string {
  const timeSymbol = world.time.symbol ?? "t";
  const stateVars = world.variables.filter((variable) => variable.role === "state").map((variable) => variable.id);
  const states = stateVars.length > 0 ? stateVars : world.variables.map((variable) => variable.id);
  const params = world.parameters ?? [];

  const paramLines = params.length === 0
    ? ["PARAMETERS: dict[str, float] = {}"]
    : ["PARAMETERS: dict[str, float] = {", ...params.map((parameter) => `    ${JSON.stringify(parameter.id)}: ${parameter.value},`), "}"];

  const unpack: string[] = [
    "    p = {**PARAMETERS, **(parameters or {})}",
    ...params.map((parameter) => `    ${parameter.id} = p[${JSON.stringify(parameter.id)}]`),
    ...states.map((id) => `    ${id} = state[${JSON.stringify(id)}]`),
  ];

  const continuous = world.laws.filter((law): law is typeof law & { target: string } => "target" in law && law.kind === "continuous");
  const otherLaws = world.laws.filter((law) => !("target" in law && law.kind === "continuous"));

  const parts: string[] = [
    `"""Standalone LawSynth world: ${world.name ?? world.id}.`,
    "",
    "Generated by LawSynth Studio (export). Zero third-party dependencies.",
    '"""',
    "import math",
    "",
    ...paramLines,
    "",
    `STATE_VARIABLES: list[str] = [${states.map((id) => JSON.stringify(id)).join(", ")}]`,
    "",
    `def derivatives(state: dict[str, float], ${timeSymbol}: float = 0.0, parameters: dict[str, float] | None = None) -> dict[str, float]:`,
    '    """Instantaneous rate of change for each continuous state variable."""',
    ...unpack,
  ];
  if (continuous.length === 0) parts.push("    return {}");
  else {
    parts.push("    return {");
    parts.push(...continuous.map((law) => `        ${JSON.stringify(law.target)}: ${expressionToPython(law.expression)},`));
    parts.push("    }");
  }

  if (otherLaws.length > 0) {
    parts.push(
      "",
      "def algebraic(state: dict[str, float], parameters: dict[str, float] | None = None) -> dict[str, float]:",
      '    """Discrete/algebraic laws evaluated from the current state."""',
      ...unpack,
      "    return {",
      ...otherLaws.map((law) => `        ${JSON.stringify("target" in law ? law.target : law.id)}: ${expressionToPython(law.expression)},`),
      "    }",
    );
  }

  parts.push("");
  return parts.join("\n");
}
