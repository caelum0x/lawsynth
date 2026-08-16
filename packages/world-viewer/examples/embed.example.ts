import { buildEmbedUrl } from "../src/index.js";
console.log(buildEmbedUrl("https://studio.example.org/share", { worldId: "sir", revision: 3, theme: "dark", panel: "trajectory", readOnly: true }));
