import { runAnalysisTests } from "./analysis.test.js";
import { runEventTests } from "./event.test.js";
import { runExpressionTests } from "./expression.test.js";
import { runGraphTests } from "./graph.test.js";
import { runInterventionTests } from "./intervention.test.js";
import { runLawTests } from "./law.test.js";
import { runManifestTests } from "./manifest.test.js";
import { runRegimeTests } from "./regime.test.js";
import { runTypesTests } from "./types.test.js";
import { runValidatorTests } from "./validators.test.js";
import { runWorldTests } from "./world.test.js";

for (const test of [runTypesTests, runExpressionTests, runGraphTests, runLawTests, runEventTests, runInterventionTests, runRegimeTests, runManifestTests, runWorldTests, runValidatorTests, runAnalysisTests]) test();
