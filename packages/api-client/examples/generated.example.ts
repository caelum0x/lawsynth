import { LawSynthClient } from "../src/index.js";

const baseUrl = process.env.LAWSYNTH_API_URL;
if (!baseUrl) throw new Error("Set LAWSYNTH_API_URL to query the deployed API version");
console.log(await new LawSynthClient({ baseUrl }).version());
