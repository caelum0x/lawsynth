import { LawSynthClient } from "../src/index.js";

const baseUrl = process.env.LAWSYNTH_API_URL;
const projectId = process.env.LAWSYNTH_PROJECT_ID;
if (!baseUrl || !projectId) throw new Error("Set LAWSYNTH_API_URL and LAWSYNTH_PROJECT_ID to create an upload session");
const session = await new LawSynthClient({ baseUrl }).uploads.create(projectId, "observations.parquet", 1024, "upload-session-0001");
console.log(session.id, session.part_size);
