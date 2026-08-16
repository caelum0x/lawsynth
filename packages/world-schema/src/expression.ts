import type { Identifier, JsonValue, SourceSpan, ValueType } from "./types.js";

export type UnaryOperator = "neg" | "abs" | "exp" | "log" | "sqrt" | "sin" | "cos" | "tan";
export type BinaryOperator = "add" | "sub" | "mul" | "div" | "pow" | "min" | "max";
export type ComparisonOperator = "eq" | "ne" | "lt" | "lte" | "gt" | "gte";
export type LogicalOperator = "and" | "or";

interface ExpressionBase {
  valueType?: ValueType;
  unit?: string;
  sourceSpan?: SourceSpan;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export type Expression =
  | (ExpressionBase & { kind: "constant"; value: number | boolean | string })
  | (ExpressionBase & { kind: "symbol"; id: Identifier })
  | (ExpressionBase & { kind: "unary"; operator: UnaryOperator; operand: Expression })
  | (ExpressionBase & { kind: "binary"; operator: BinaryOperator; left: Expression; right: Expression })
  | (ExpressionBase & { kind: "comparison"; operator: ComparisonOperator; left: Expression; right: Expression })
  | (ExpressionBase & { kind: "logical"; operator: LogicalOperator; operands: readonly Expression[] })
  | (ExpressionBase & { kind: "not"; operand: Expression })
  | (ExpressionBase & { kind: "delay"; expression: Expression; lag: number })
  | (ExpressionBase & { kind: "piecewise"; branches: readonly PiecewiseBranch[]; otherwise: Expression })
  | (ExpressionBase & { kind: "call"; function: Identifier; arguments: readonly Expression[] });

export interface PiecewiseBranch {
  when: Expression;
  then: Expression;
}

export type EvaluationScope = Readonly<Record<Identifier, number | boolean | string>>;

export function collectSymbols(expression: Expression): ReadonlySet<Identifier> {
  const symbols = new Set<Identifier>();
  visitExpression(expression, (node) => {
    if (node.kind === "symbol") symbols.add(node.id);
  });
  return symbols;
}

export function visitExpression(expression: Expression, visitor: (node: Expression) => void): void {
  const stack: Expression[] = [expression];
  while (stack.length > 0) {
    const node = stack.pop();
    if (!node) continue;
    visitor(node);
    switch (node.kind) {
      case "unary":
      case "not":
        stack.push(node.operand);
        break;
      case "binary":
      case "comparison":
        stack.push(node.right, node.left);
        break;
      case "logical":
        stack.push(...[...node.operands].reverse());
        break;
      case "delay":
        stack.push(node.expression);
        break;
      case "piecewise":
        stack.push(node.otherwise);
        for (const branch of [...node.branches].reverse()) stack.push(branch.then, branch.when);
        break;
      case "call":
        stack.push(...[...node.arguments].reverse());
        break;
      case "constant":
      case "symbol":
        break;
    }
  }
}

export function expressionDepth(expression: Expression): number {
  switch (expression.kind) {
    case "constant":
    case "symbol":
      return 1;
    case "unary":
    case "not":
      return 1 + expressionDepth(expression.operand);
    case "delay":
      return 1 + expressionDepth(expression.expression);
    case "binary":
    case "comparison":
      return 1 + Math.max(expressionDepth(expression.left), expressionDepth(expression.right));
    case "logical":
      return 1 + Math.max(0, ...expression.operands.map(expressionDepth));
    case "call":
      return 1 + Math.max(0, ...expression.arguments.map(expressionDepth));
    case "piecewise":
      return 1 + Math.max(
        expressionDepth(expression.otherwise),
        ...expression.branches.flatMap((branch) => [expressionDepth(branch.when), expressionDepth(branch.then)]),
      );
  }
}

export function evaluateExpression(expression: Expression, scope: EvaluationScope): number | boolean | string {
  switch (expression.kind) {
    case "constant":
      return expression.value;
    case "symbol": {
      const value = scope[expression.id];
      if (value === undefined) throw new Error(`Missing expression symbol: ${expression.id}`);
      return value;
    }
    case "unary":
      return evaluateUnary(expression.operator, requireNumber(evaluateExpression(expression.operand, scope)));
    case "binary":
      return evaluateBinary(
        expression.operator,
        requireNumber(evaluateExpression(expression.left, scope)),
        requireNumber(evaluateExpression(expression.right, scope)),
      );
    case "comparison":
      return compare(expression.operator, evaluateExpression(expression.left, scope), evaluateExpression(expression.right, scope));
    case "logical": {
      const values = expression.operands.map((operand) => requireBoolean(evaluateExpression(operand, scope)));
      return expression.operator === "and" ? values.every(Boolean) : values.some(Boolean);
    }
    case "not":
      return !requireBoolean(evaluateExpression(expression.operand, scope));
    case "piecewise": {
      const branch = expression.branches.find((candidate) =>
        requireBoolean(evaluateExpression(candidate.when, scope)),
      );
      return evaluateExpression(branch?.then ?? expression.otherwise, scope);
    }
    case "delay":
      throw new Error("Delay expressions require a history-aware evaluator");
    case "call":
      throw new Error(`Custom function ${expression.function} requires a registered evaluator`);
  }
}

function evaluateUnary(operator: UnaryOperator, value: number): number {
  const result = {
    neg: -value,
    abs: Math.abs(value),
    exp: Math.exp(value),
    log: Math.log(value),
    sqrt: Math.sqrt(value),
    sin: Math.sin(value),
    cos: Math.cos(value),
    tan: Math.tan(value),
  }[operator];
  if (!Number.isFinite(result)) throw new RangeError(`Unary operator ${operator} produced a non-finite value`);
  return result;
}

function evaluateBinary(operator: BinaryOperator, left: number, right: number): number {
  const result = {
    add: left + right,
    sub: left - right,
    mul: left * right,
    div: left / right,
    pow: left ** right,
    min: Math.min(left, right),
    max: Math.max(left, right),
  }[operator];
  if (!Number.isFinite(result)) throw new RangeError(`Binary operator ${operator} produced a non-finite value`);
  return result;
}

function compare(operator: ComparisonOperator, left: number | boolean | string, right: number | boolean | string): boolean {
  if (typeof left !== typeof right) throw new TypeError("Comparison operands must have the same type");
  switch (operator) {
    case "eq": return left === right;
    case "ne": return left !== right;
    case "lt": return requireOrdered(left) < requireOrdered(right);
    case "lte": return requireOrdered(left) <= requireOrdered(right);
    case "gt": return requireOrdered(left) > requireOrdered(right);
    case "gte": return requireOrdered(left) >= requireOrdered(right);
  }
}

function requireOrdered(value: number | boolean | string): number | string {
  if (typeof value === "boolean") throw new TypeError("Booleans only support equality comparisons");
  return value;
}

function requireNumber(value: number | boolean | string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new TypeError("Expected a finite number");
  return value;
}

function requireBoolean(value: number | boolean | string): boolean {
  if (typeof value !== "boolean") throw new TypeError("Expected a boolean");
  return value;
}
