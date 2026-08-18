/**
 * Engine analysis reports (`stability`, `control`, `domains run`).
 *
 * These reports are produced by the `lawsynth` CLI's `--json` mode, not by the
 * LawSynth Server HTTP contract — there is no analysis endpoint to `GET`. The
 * public surface here is therefore a set of pure PARSERS that validate the
 * engine's `unknown` JSON and narrow it into the typed models from
 * `@lawsynth/world-schema`. Feed them the parsed body of a CLI artifact (or a
 * downloaded analysis file) to get a checked, typed result:
 *
 * ```ts
 * import { parseStabilityReport } from "@lawsynth/api-client";
 * const report = parseStabilityReport(JSON.parse(cliOutput));
 * ```
 *
 * The types and parsers are re-exported unchanged from `@lawsynth/world-schema`
 * so callers can consume them directly from the client package.
 */

export {
  BIFURCATION_KINDS,
  CLASSIFICATIONS,
  OBSERVER_METHODS,
  parseBasinReport,
  parseBifurcationReport,
  parseControlledModel,
  parseDomainRun,
  parseEstimateReport,
  parseLyapunovReport,
  parseMpcResult,
  parseNetworkModel,
  parseReductionReport,
  parseSensitivityReport,
  parseStabilityReport,
  validateBasinReport,
  validateBifurcationReport,
  validateControlledModel,
  validateDomainRun,
  validateEstimateReport,
  validateLyapunovReport,
  validateMpcResult,
  validateNetworkModel,
  validateReductionReport,
  validateSensitivityReport,
  validateStabilityReport,
} from "@lawsynth/world-schema";

export type {
  Attractor,
  BasinLabel,
  BasinReport,
  Bifurcation,
  BifurcationKind,
  BifurcationRange,
  BifurcationReport,
  Classification,
  ControlEquation,
  ControlPerStateScore,
  ControlTerm,
  ControlValidation,
  ControlledModel,
  DomainRecovery,
  DomainRunReport,
  Eigenvalue,
  EstimateReport,
  FixedPoint,
  LyapunovReport,
  Matrix,
  MpcResult,
  NetworkEdge,
  NetworkModel,
  ObserverMethod,
  ReducedSystem,
  ReductionReport,
  SensitivityEntry,
  SensitivityReport,
  StabilityReport,
} from "@lawsynth/world-schema";
