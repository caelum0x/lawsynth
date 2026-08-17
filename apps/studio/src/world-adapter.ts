import type { WorldRecord } from "@lawsynth/api-client";
import type { Expression, Law, ParameterDefinition, VariableDefinition, WorldDefinition } from "@lawsynth/world-schema";

/**
 * Adapts a live discovery result — the declarative {@link WorldRecord} returned
 * by `GET /v1/runs/{id}/world` (states, parameters, and one arithmetic
 * expression per state) — into the rich {@link WorldDefinition} IR the Studio
 * screens (equation explorer, structure map, …) render. The heart of it is a
 * small, dependency-free arithmetic parser that turns an equation string such as
 * `-4*x - 0.5*v` into the schema's {@link Expression} AST.
 */

const UNARY_FUNCTIONS: Readonly<Record<string, "abs" | "exp" | "log" | "sqrt" | "sin" | "cos" | "tan">> = {
  abs: "abs", exp: "exp", log: "log", ln: "log", sqrt: "sqrt", sin: "sin", cos: "cos", tan: "tan",
};

type Token =
  | { readonly kind: "number"; readonly value: number }
  | { readonly kind: "name"; readonly value: string }
  | { readonly kind: "op"; readonly value: "+" | "-" | "*" | "/" | "**" | "^" | "(" | ")" | "," };

function tokenize(source: string): readonly Token[] {
  const tokens: Token[] = [];
  let i = 0;
  const isDigit = (c: string): boolean => c >= "0" && c <= "9";
  const isNameStart = (c: string): boolean => (c >= "A" && c <= "Z") || (c >= "a" && c <= "z") || c === "_";
  const isNamePart = (c: string): boolean => isNameStart(c) || isDigit(c);
  while (i < source.length) {
    const c = source[i]!;
    if (c === " " || c === "\t" || c === "\n" || c === "\r") { i += 1; continue; }
    if (c === "*" && source[i + 1] === "*") { tokens.push({ kind: "op", value: "**" }); i += 2; continue; }
    if (c === "+" || c === "-" || c === "*" || c === "/" || c === "^" || c === "(" || c === ")" || c === ",") {
      tokens.push({ kind: "op", value: c }); i += 1; continue;
    }
    if (isDigit(c) || (c === "." && isDigit(source[i + 1] ?? ""))) {
      let j = i + 1;
      while (j < source.length && (isDigit(source[j]!) || source[j] === ".")) j += 1;
      if (source[j] === "e" || source[j] === "E") {
        j += 1;
        if (source[j] === "+" || source[j] === "-") j += 1;
        while (j < source.length && isDigit(source[j]!)) j += 1;
      }
      const value = Number(source.slice(i, j));
      if (!Number.isFinite(value)) throw new RangeError(`invalid number in expression: ${source.slice(i, j)}`);
      tokens.push({ kind: "number", value }); i = j; continue;
    }
    if (isNameStart(c)) {
      let j = i + 1;
      while (j < source.length && isNamePart(source[j]!)) j += 1;
      tokens.push({ kind: "name", value: source.slice(i, j) }); i = j; continue;
    }
    throw new RangeError(`unexpected character ${JSON.stringify(c)} in expression`);
  }
  return tokens;
}

class Parser {
  #pos = 0;
  constructor(private readonly tokens: readonly Token[]) {}

  parse(): Expression {
    const expression = this.#expr();
    if (this.#pos !== this.tokens.length) throw new RangeError("trailing tokens in expression");
    return expression;
  }

  #peek(): Token | undefined { return this.tokens[this.#pos]; }
  #next(): Token | undefined { return this.tokens[this.#pos++]; }

  #expr(): Expression {
    let left = this.#term();
    for (let token = this.#peek(); token?.kind === "op" && (token.value === "+" || token.value === "-"); token = this.#peek()) {
      this.#pos += 1;
      const right = this.#term();
      left = { kind: "binary", operator: token.value === "+" ? "add" : "sub", left, right };
    }
    return left;
  }

  #term(): Expression {
    let left = this.#factor();
    for (let token = this.#peek(); token?.kind === "op" && (token.value === "*" || token.value === "/"); token = this.#peek()) {
      this.#pos += 1;
      const right = this.#factor();
      left = { kind: "binary", operator: token.value === "*" ? "mul" : "div", left, right };
    }
    return left;
  }

  #factor(): Expression {
    const token = this.#peek();
    if (token?.kind === "op" && (token.value === "+" || token.value === "-")) {
      this.#pos += 1;
      const operand = this.#factor();
      return token.value === "-" ? { kind: "unary", operator: "neg", operand } : operand;
    }
    return this.#power();
  }

  #power(): Expression {
    const base = this.#primary();
    const token = this.#peek();
    if (token?.kind === "op" && (token.value === "**" || token.value === "^")) {
      this.#pos += 1;
      const exponent = this.#factor(); // right-associative, binds tighter than unary on the right
      return { kind: "binary", operator: "pow", left: base, right: exponent };
    }
    return base;
  }

  #primary(): Expression {
    const token = this.#next();
    if (token === undefined) throw new RangeError("unexpected end of expression");
    if (token.kind === "number") return { kind: "constant", value: token.value };
    if (token.kind === "op" && token.value === "(") {
      const inner = this.#expr();
      this.#expect(")");
      return inner;
    }
    if (token.kind === "name") {
      const next = this.#peek();
      if (next?.kind === "op" && next.value === "(") {
        this.#pos += 1;
        const args = this.#arguments();
        this.#expect(")");
        return this.#callExpression(token.value, args);
      }
      return { kind: "symbol", id: token.value };
    }
    throw new RangeError(`unexpected token ${JSON.stringify(token.value)} in expression`);
  }

  #arguments(): readonly Expression[] {
    const args: Expression[] = [];
    if (this.#peek()?.kind === "op" && (this.#peek() as { value: string }).value === ")") return args;
    args.push(this.#expr());
    for (let token = this.#peek(); token?.kind === "op" && token.value === ","; token = this.#peek()) {
      this.#pos += 1;
      args.push(this.#expr());
    }
    return args;
  }

  #callExpression(name: string, args: readonly Expression[]): Expression {
    const unary = UNARY_FUNCTIONS[name];
    if (unary !== undefined && args.length === 1) return { kind: "unary", operator: unary, operand: args[0]! };
    if ((name === "min" || name === "max") && args.length === 2) return { kind: "binary", operator: name, left: args[0]!, right: args[1]! };
    return { kind: "call", function: name, arguments: args };
  }

  #expect(value: ")"): void {
    const token = this.#next();
    if (token === undefined || token.kind !== "op" || token.value !== value) throw new RangeError(`expected ${value} in expression`);
  }
}

/** Parse a plain arithmetic expression string into the schema Expression AST. */
export function parseArithmetic(source: string): Expression {
  const trimmed = source.trim();
  if (trimmed === "") return { kind: "constant", value: 0 };
  return new Parser(tokenize(trimmed)).parse();
}

/** Build a full {@link WorldDefinition} from a live discovery {@link WorldRecord}. */
export function worldFromRecord(record: WorldRecord): WorldDefinition {
  const states = record.states ?? [];
  const controls = record.controls ?? [];
  const variables: readonly VariableDefinition[] = Object.freeze([
    ...states.map((id): VariableDefinition => ({ id, role: "state" })),
    ...controls.map((id): VariableDefinition => ({ id, role: "control" })),
  ]);
  const parameters: readonly ParameterDefinition[] = Object.freeze(
    Object.entries(record.parameters ?? {}).map(([id, value]): ParameterDefinition => ({ id, value })),
  );
  const laws: readonly Law[] = Object.freeze(
    Object.keys(record.equations ?? {})
      .sort()
      .map((target): Law => ({
        id: `law_${target}`,
        kind: "continuous",
        target,
        expression: parseArithmetic(record.equations[target] ?? ""),
        enabled: true,
      })),
  );
  return {
    formatVersion: "0.1.0",
    id: record.id,
    ...(record.name ? { name: record.name } : {}),
    time: { kind: "continuous", symbol: "t" },
    variables,
    ...(parameters.length > 0 ? { parameters } : {}),
    laws,
  };
}
