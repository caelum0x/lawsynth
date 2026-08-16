import { createShareUrl, parseShareUrl } from "../src/share.js";
import { PlaygroundError } from "../src/errors.js";
import { decayWorld, deepEqual, equal, test, throws } from "./testkit.js";

await test("share URLs round-trip the actual world and finite parameter overrides", () => {
  const url = createShareUrl("https://lawsynth.example/play", { version: 1, world: decayWorld, parameters: { rate: 0.4 } });
  const parsed = parseShareUrl(url);
  equal(parsed?.world.id, decayWorld.id);
  deepEqual(parsed?.parameters, { rate: 0.4 });
  equal(parseShareUrl("https://lawsynth.example/play"), undefined);
  throws(() => parseShareUrl("https://lawsynth.example/play#world=%%%"), /share link is malformed/);
  throws(() => createShareUrl("https://lawsynth.example/play", { version: 1, world: decayWorld, parameters: { rate: Number.NaN } }), /finite/);
  equal(new PlaygroundError("share-failed", "x").code, "share-failed");
});
