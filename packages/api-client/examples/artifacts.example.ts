import { LawSynthClient } from "../src/index.js";

const baseUrl = process.env.LAWSYNTH_API_URL;
const artifactId = process.env.LAWSYNTH_ARTIFACT_ID;
if (!baseUrl || !artifactId) throw new Error("Set LAWSYNTH_API_URL and LAWSYNTH_ARTIFACT_ID to download an actual artifact");
const bytes = await new LawSynthClient({ baseUrl }).downloads.bytes(artifactId);
console.log(`downloaded ${bytes.byteLength} bytes`);
