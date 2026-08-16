import { createProgress } from "../src/index.js";

const progress = createProgress({ label: "Evaluating candidate equations", value: 32, max: 50 });
console.log(JSON.stringify(progress));
