import { createEvent, createUndoStack, pushUndo, takeUndo } from "../src/index.js";

const forward = createEvent({ type: "selection.set", ids: ["variable:x"] }, "e:1", 1);
const inverse = createEvent({ type: "selection.set", ids: [] }, "e:2", 2);
const stack = pushUndo(createUndoStack(), { event: forward, inverse });
console.log(takeUndo(stack).event);
