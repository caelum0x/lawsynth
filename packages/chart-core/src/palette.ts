const PALETTE = ["#2563eb", "#dc2626", "#059669", "#9333ea", "#ea580c", "#0891b2", "#4f46e5", "#be123c"] as const;

/** Stable categorical colors. A series keeps its color when neighboring series change. */
export function categoricalColor(id: string): string {
  if (!id) throw new TypeError("color id must be non-empty");
  let hash = 2166136261;
  for (let i = 0; i < id.length; i += 1) { hash ^= id.charCodeAt(i); hash = Math.imul(hash, 16777619); }
  return PALETTE[(hash >>> 0) % PALETTE.length]!;
}

export function sequentialColor(value: number, domain: readonly [number, number]): string {
  const [min, max] = domain;
  if (!Number.isFinite(value) || !Number.isFinite(min) || !Number.isFinite(max) || max <= min) throw new RangeError("sequential color requires a non-empty finite domain");
  const ratio = Math.max(0, Math.min(1, (value - min) / (max - min)));
  const red = Math.round(25 + ratio * 214); const green = Math.round(55 + ratio * 120); const blue = Math.round(109 + ratio * 25);
  return `rgb(${red}, ${green}, ${blue})`;
}
