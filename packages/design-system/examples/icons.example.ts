import { createIcon } from "../src/index.js";

const warning = createIcon("warning", { label: "Warning" });
const decorativeChevron = createIcon("chevronDown", { decorative: true });
console.log(JSON.stringify([warning, decorativeChevron]));
