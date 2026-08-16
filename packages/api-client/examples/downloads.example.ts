import { LawSynthClient } from "../src/index.js";

const baseUrl = process.env.LAWSYNTH_API_URL;
const artifactId = process.env.LAWSYNTH_ARTIFACT_ID;
if (!baseUrl || !artifactId) throw new Error("Set LAWSYNTH_API_URL and LAWSYNTH_ARTIFACT_ID to retrieve an artifact");
const artifact = await new LawSynthClient({ baseUrl }).artifacts.get(artifactId);
console.log(`${artifact.id}: ${artifact.byte_len} bytes (${artifact.sha256})`);
