import assert from "node:assert/strict";
import test from "node:test";
import { expressionSymbols, formatExpression } from "../src/index.js";
const expression = { kind: "piecewise", branches: [{ when: { kind: "comparison", operator: "gt", left: { kind: "symbol", id: "x" }, right: { kind: "constant", value: 0 } }, then: { kind: "call", function: "exp", arguments: [{ kind: "symbol", id: "x" }] } }], otherwise: { kind: "constant", value: 0 } } as const;
test("formats structured equations and collects symbols", () => { assert.match(formatExpression(expression), /piecewise/); assert.deepEqual(expressionSymbols(expression), ["x"]); });
