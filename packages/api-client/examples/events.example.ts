import { LawSynthClient } from "../src/index.js";

const baseUrl = process.env.LAWSYNTH_API_URL;
const runId = process.env.LAWSYNTH_RUN_ID;
if (!baseUrl || !runId) throw new Error("Set LAWSYNTH_API_URL and LAWSYNTH_RUN_ID to consume a live run event stream");
for await (const event of new LawSynthClient({ baseUrl }).events(runId)) console.log(event.topic, event.occurred_at);
