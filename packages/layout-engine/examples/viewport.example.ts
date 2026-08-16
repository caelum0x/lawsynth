import { createViewport, fitBounds, screenToWorld, zoomAt } from "../src/index.js";

const initial = createViewport(1280, 800);
const fitted = fitBounds(initial, { x: -120, y: -40, width: 640, height: 400 });
const zoomed = zoomAt(fitted, 1.25, { x: 640, y: 400 });
export const pointerInWorld = screenToWorld(zoomed, { x: 640, y: 400 });
