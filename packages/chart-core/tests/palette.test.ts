import { equal, throws } from "./assert.js";
import { categoricalColor, sequentialColor } from "../src/palette.js";

equal(categoricalColor("velocity"), categoricalColor("velocity"));
equal(sequentialColor(5, [0, 10]).startsWith("rgb("), true);
throws(() => sequentialColor(1, [1, 1]));
