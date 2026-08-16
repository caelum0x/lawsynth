import { WorldEditor, parseWorldText } from "../src/editor.js";
import { decayWorld, equal, ok, test } from "./testkit.js";

await test("editor parses executable world JSON and reports source diagnostics", () => {
  const parsed = parseWorldText(JSON.stringify(decayWorld));
  equal(parsed.diagnostics.length, 0);
  equal(parsed.world?.id, decayWorld.id);

  const malformed = parseWorldText('{\n  "formatVersion":\n');
  equal(malformed.diagnostics[0]?.severity, "error");
  ok(malformed.diagnostics[0]?.line !== undefined);
});

await test("editor validation commits the latest revision and saved state", () => {
  const editor = new WorldEditor("{}", 60_000);
  editor.update(JSON.stringify(decayWorld));
  const valid = editor.validate();
  equal(valid.world?.id, decayWorld.id);
  equal(valid.dirty, true);
  editor.markSaved();
  equal(editor.snapshot.dirty, false);
  editor.load(decayWorld);
  equal(editor.snapshot.dirty, false);
  equal(editor.snapshot.diagnostics.length, 0);
  editor.dispose();
});
