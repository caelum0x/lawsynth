import type { Domain } from "./scales.js";

export function zoomDomain(domain: Domain, factor: number, anchor: number): Domain {
  if (!Number.isFinite(domain.min) || !Number.isFinite(domain.max) || domain.max <= domain.min) throw new RangeError("zoom needs a non-empty finite domain");
  if (!Number.isFinite(factor) || factor <= 0) throw new RangeError("zoom factor must be positive");
  if (!Number.isFinite(anchor)) throw new RangeError("zoom anchor must be finite");
  const span = (domain.max - domain.min) / factor;
  const ratio = (anchor - domain.min) / (domain.max - domain.min);
  return { min: anchor - span * ratio, max: anchor + span * (1 - ratio) };
}

export function panDomain(domain: Domain, delta: number): Domain {
  if (!Number.isFinite(delta)) throw new RangeError("pan delta must be finite");
  return { min: domain.min + delta, max: domain.max + delta };
}
