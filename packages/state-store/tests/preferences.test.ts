import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_PREFERENCES, updatePreferences } from "../src/preferences.js";

test("preferences accept only declared values", () => {
  assert.equal(updatePreferences(DEFAULT_PREFERENCES, { theme: "dark", reducedMotion: true }).theme, "dark");
  assert.throws(() => updatePreferences(DEFAULT_PREFERENCES, { theme: "neon" as never }), /theme/);
});
