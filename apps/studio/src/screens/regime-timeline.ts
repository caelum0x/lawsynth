import { categoricalColor, defaultTickFormatter, makeAxisTicks, type AxisSpec } from "@lawsynth/chart-core";
import { timelineLayout, type TimelineEvent } from "@lawsynth/layout-engine";
import type { RegimeInterval, WorldDefinition } from "@lawsynth/world-schema";
import { regimeTimelineForWorld } from "@lawsynth/world-viewer";
import type { Metric, Notice, ScreenModel, ScreenSection, TimelineBoundary, TimelineSegment, TimelineView, TimelineAxisTick } from "./types.js";

export interface RegimeTimelineInput {
  readonly world: WorldDefinition;
  readonly targetWidth?: number;
  readonly selectedRegime?: string;
}

const LANE_HEIGHT = 40;
const LANE_GAP = 10;

function regimeLabel(world: WorldDefinition, regimeId: string): string {
  return world.regimes?.regimes.find((definition) => definition.id === regimeId)?.name ?? regimeId;
}

/**
 * Builds a pixel-space regime timeline by composing the layout engine's
 * `timelineLayout` for segment geometry with `chart-core` axis ticks for the
 * time scale. The segment/tick/boundary mappings all share the layout engine's
 * `x = (t - origin) * scale` transform so they line up exactly.
 */
export function regimeTimelineView(input: RegimeTimelineInput): TimelineView | undefined {
  const { world } = input;
  const summary = regimeTimelineForWorld(world);
  const intervals = world.regimes?.intervals ?? [];
  if (summary === undefined || intervals.length === 0) return undefined;

  const start = summary.start;
  const end = summary.end;
  const span = end - start || 1;
  const targetWidth = input.targetWidth ?? 760;
  const scale = targetWidth / span;
  const toX = (time: number): number => (time - start) * scale;

  const events: readonly TimelineEvent[] = intervals.map((interval: RegimeInterval, index) => ({
    id: `seg-${index}`,
    start: interval.start,
    end: interval.end,
    lane: interval.regime,
  }));
  const items = timelineLayout(events, { pixelsPerUnit: scale, laneHeight: LANE_HEIGHT, laneGap: LANE_GAP, origin: start });

  const segments: readonly TimelineSegment[] = items.map((item, index) => {
    const interval = intervals[index]!;
    const selected = input.selectedRegime === interval.regime;
    return {
      id: item.id,
      regime: interval.regime,
      label: regimeLabel(world, interval.regime),
      start: interval.start,
      end: interval.end,
      x: item.x,
      y: item.y,
      width: item.width,
      height: item.height,
      color: categoricalColor(interval.regime),
      selected,
      ...(interval.confidence === undefined ? {} : { confidence: interval.confidence }),
    };
  });

  const boundaryTimes = [...new Set(intervals.flatMap((interval) => [interval.start, interval.end]))].sort((a, b) => a - b);
  const boundaries: readonly TimelineBoundary[] = boundaryTimes.map((time, index) => ({
    id: `bound-${index}`,
    time,
    x: toX(time),
    label: defaultTickFormatter(time),
  }));

  const axis: AxisSpec = { domain: { min: start, max: end }, label: world.time.symbol ?? "t", tickCount: 6 };
  const ticks: readonly TimelineAxisTick[] = makeAxisTicks(axis).map((tick) => ({ value: tick.value, x: toX(tick.value), label: tick.label }));

  const laneCount = new Set(intervals.map((interval) => interval.regime)).size;
  const height = laneCount * (LANE_HEIGHT + LANE_GAP);
  return { segments, boundaries, ticks, width: targetWidth, height, start, end };
}

export function regimeTimelineModel(input: RegimeTimelineInput): ScreenModel {
  const view = regimeTimelineView(input);
  const sections: ScreenSection[] = [];

  if (view === undefined) {
    const notices: readonly Notice[] = [{ tone: "info", message: "This world has no discovered regime segments to display." }];
    sections.push({ kind: "notices", id: "no-regimes", notices });
    return { id: "regime-timeline", title: "Regime Timeline", subtitle: "Discovered regime segments over time", sections };
  }

  const intervals = input.world.regimes?.intervals ?? [];
  const confidences = intervals.map((interval) => interval.confidence).filter((value): value is number => value !== undefined);
  const meanConfidence = confidences.length === 0 ? undefined : confidences.reduce((sum, value) => sum + value, 0) / confidences.length;
  const metrics: readonly Metric[] = [
    { label: "Regimes", value: String(new Set(intervals.map((interval) => interval.regime)).size) },
    { label: "Segments", value: String(intervals.length) },
    { label: "Span", value: `${view.start} → ${view.end}` },
    { label: "Mean confidence", value: meanConfidence === undefined ? "—" : `${Math.round(meanConfidence * 100)}%` },
  ];

  sections.push({ kind: "metrics", id: "regime-metrics", title: "Regimes", metrics });
  sections.push({ kind: "timeline", id: "timeline", title: "Timeline", timeline: view });
  return { id: "regime-timeline", title: "Regime Timeline", subtitle: "Discovered regime segments over time", sections };
}
