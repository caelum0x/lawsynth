import { createToast } from "../src/index.js";

const toast = createToast({ id: "bundle-created", message: "Bundle created", tone: "success", dismissAction: "toast.dismiss", timeoutMs: 6000 });
console.log(JSON.stringify(toast));
