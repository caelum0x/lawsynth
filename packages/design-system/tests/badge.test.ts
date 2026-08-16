import { assert, test } from "./assertions.js";
import { createBadge } from "../src/badge.js";
import { assertContrast, defaultTokens, validateDesignTokens } from "../src/tokens.js";
import { createTheme } from "../src/theme.js";

test("badge can opt into polite announcements", () => {
  assert.equal(createBadge({ label: "Ready", tone: "success", announce: true }).attributes["aria-live"], "polite");
});

test("tokens enforce accessible color contrast", () => {
  validateDesignTokens(defaultTokens);
  assert.throws(() => assertContrast("#777777", "#ffffff", 4.5), /insufficient/);
  assert.throws(() => createTheme("bad", { ...defaultTokens, color: { ...defaultTokens.color, text: "#999999" } }), /insufficient/);
});
