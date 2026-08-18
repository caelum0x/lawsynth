import { analysisTests } from "./analysis.test.js";
import { analysisViewTests } from "./analysis_view.test.js";
import { appTests } from "./app.test.js";
import { collaborationTests } from "./collaboration.test.js";
import { dataPrepTests } from "./data-prep.test.js";
import { datasetTests } from "./dataset.test.js";
import { discoveryTests } from "./discovery.test.js";
import { equationsTests } from "./equations.test.js";
import { monitorTests } from "./monitor.test.js";
import { providersTests } from "./providers.test.js";
import { regimesTests } from "./regimes.test.js";
import { routesTests } from "./routes.test.js";
import { scenarioBoardTests } from "./scenario-board.test.js";
import { simulationTests } from "./simulation.test.js";
import { structureTests } from "./structure.test.js";
import { wiredDiscoveryTests } from "./wired-discovery.test.js";
import { workspaceTests } from "./workspace.test.js";

const tests: readonly [string, () => Promise<void>][] = [
  ["analysis view models", analysisTests], ["analysis rendering", analysisViewTests], ["app lifecycle", appTests], ["collaboration", collaborationTests], ["data prep", dataPrepTests], ["dataset profiling", datasetTests], ["discovery observation", discoveryTests],
  ["equation comparison", equationsTests], ["monitor", monitorTests], ["provider lifecycle", providersTests], ["regime planning", regimesTests],
  ["routes", routesTests], ["scenario board", scenarioBoardTests], ["simulation execution", simulationTests], ["structure filtering", structureTests],
  ["wired discovery", wiredDiscoveryTests], ["workspace selection", workspaceTests],
];

for (const [name, run] of tests) {
  await run();
  console.log(`ok - ${name}`);
}
