import type { RegimeInterval, RegimeModel, WorldDefinition } from "@lawsynth/world-schema";
import { timelineLayout, type TimelineItem } from "@lawsynth/layout-engine";

export interface RegimeLane extends TimelineItem {
  readonly regime: string;
  readonly label: string;
  readonly confidence?: number;
  readonly colorIndex: number;
}

export interface RegimeTimeline {
  readonly lanes: readonly RegimeLane[];
  readonly start: number;
  readonly end: number;
  readonly regimeCount: number;
}

export function regimeTimeline(model: RegimeModel | undefined, width = 720): RegimeTimeline | undefined {
  if (model === undefined || model.intervals === undefined || model.intervals.length === 0) return undefined;
  if (!Number.isFinite(width) || width <= 0) throw new RangeError("timeline width must be positive");
  const definitions = new Map(model.regimes.map((regime) => [regime.id, regime]));
  const ids = new Set<string>();
  let start = Infinity;
  let end = -Infinity;
  model.intervals.forEach((interval, index) => {
    if (!definitions.has(interval.regime)) throw new RangeError(`unknown regime interval: ${interval.regime}`);
    if (![interval.start, interval.end].every(Number.isFinite) || interval.end <= interval.start) throw new RangeError(`regime interval ${index} must be finite and non-empty`);
    if (interval.confidence !== undefined && (!Number.isFinite(interval.confidence) || interval.confidence < 0 || interval.confidence > 1)) throw new RangeError(`regime interval ${index} confidence must be in [0, 1]`);
    const id = `${interval.regime}:${interval.start}:${interval.end}:${index}`;
    if (ids.has(id)) throw new RangeError(`duplicate regime interval: ${id}`);
    ids.add(id);
    start = Math.min(start, interval.start);
    end = Math.max(end, interval.end);
  });
  const span = Math.max(Number.EPSILON, end - start);
  const intervalById = new Map<string, RegimeInterval>();
  const events = model.intervals.map((interval, index) => {
    const id = `${interval.regime}:${interval.start}:${interval.end}:${index}`;
    intervalById.set(id, interval);
    return {
    id,
    start: interval.start,
    end: interval.end,
    lane: "regimes",
  }; });
  const items = timelineLayout(events, { origin: start, pixelsPerUnit: width / span, laneHeight: 34, laneGap: 0 });
  const colorByRegime = new Map(model.regimes.map((regime, index) => [regime.id, index]));
  const lanes = items.map((item): RegimeLane => {
    const interval = intervalById.get(item.id);
    if (interval === undefined) throw new Error(`lost regime interval ${item.id} during layout`);
    return Object.freeze({
      ...item,
      regime: interval.regime,
      label: definitions.get(interval.regime)?.name ?? interval.regime,
      ...(interval.confidence === undefined ? {} : { confidence: interval.confidence }),
      colorIndex: colorByRegime.get(interval.regime) ?? 0,
    });
  });
  return Object.freeze({ lanes: Object.freeze(lanes), start, end, regimeCount: model.regimes.length });
}

export function regimeTimelineForWorld(world: WorldDefinition, width?: number): RegimeTimeline | undefined {
  return regimeTimeline(world.regimes, width);
}
