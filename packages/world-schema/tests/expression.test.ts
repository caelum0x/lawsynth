import { collectSymbols, evaluateExpression, expressionDepth } from "../src/expression.js";
import { equal, ok, throws } from "./test-support.js";
export function runExpressionTests(): void { const value = { kind: "binary", operator: "mul", left: { kind: "symbol", id: "x" }, right: { kind: "constant", value: 2 } } as const; equal(evaluateExpression(value, { x: 3 }), 6); equal(expressionDepth(value), 2); ok(collectSymbols(value).has("x")); throws(() => evaluateExpression({ kind: "delay", expression: value, lag: 1 }, { x: 1 })); }
