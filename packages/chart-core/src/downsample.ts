export interface XYPoint { readonly x: number; readonly y: number; }

function assertFinitePoints(points: readonly XYPoint[]): void {
  for (const [index, point] of points.entries()) {
    if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
      throw new RangeError(`point ${index} has a non-finite coordinate`);
    }
  }
}

/**
 * Largest-Triangle-Three-Buckets decimation. It preserves endpoints and favors
 * visual extrema, unlike uniform stride sampling. Input order is never changed.
 */
export function largestTriangleThreeBuckets(points: readonly XYPoint[], threshold: number): XYPoint[] {
  assertFinitePoints(points);
  if (!Number.isInteger(threshold) || threshold < 2) throw new RangeError("threshold must be an integer of at least 2");
  if (threshold >= points.length || points.length <= 2) return points.map((point) => ({ ...point }));
  if (threshold === 2) return [{ ...points[0]! }, { ...points[points.length - 1]! }];

  const sampled: XYPoint[] = [{ ...points[0]! }];
  const every = (points.length - 2) / (threshold - 2);
  let anchor = 0;
  for (let bucket = 0; bucket < threshold - 2; bucket += 1) {
    const nextStart = Math.floor((bucket + 1) * every) + 1;
    const nextEnd = Math.min(Math.floor((bucket + 2) * every) + 1, points.length);
    let averageX = 0;
    let averageY = 0;
    const averageCount = Math.max(nextEnd - nextStart, 1);
    for (let i = nextStart; i < nextEnd; i += 1) { averageX += points[i]!.x; averageY += points[i]!.y; }
    if (nextStart === nextEnd) { averageX = points[Math.min(nextStart, points.length - 1)]!.x; averageY = points[Math.min(nextStart, points.length - 1)]!.y; }
    else { averageX /= averageCount; averageY /= averageCount; }

    const rangeStart = Math.floor(bucket * every) + 1;
    const rangeEnd = Math.min(Math.floor((bucket + 1) * every) + 1, points.length - 1);
    const a = points[anchor]!;
    let maxArea = -1;
    let chosen = rangeStart;
    for (let i = rangeStart; i < rangeEnd; i += 1) {
      const point = points[i]!;
      const area = Math.abs((a.x - averageX) * (point.y - a.y) - (a.x - point.x) * (averageY - a.y));
      if (area > maxArea) { maxArea = area; chosen = i; }
    }
    sampled.push({ ...points[chosen]! });
    anchor = chosen;
  }
  sampled.push({ ...points[points.length - 1]! });
  return sampled;
}

/** Return the original precision below the configured display threshold. */
export function downsampleForViewport(points: readonly XYPoint[], pixelWidth: number, pointsPerPixel = 2): XYPoint[] {
  if (!Number.isFinite(pixelWidth) || pixelWidth <= 0) throw new RangeError("pixelWidth must be positive");
  if (!Number.isFinite(pointsPerPixel) || pointsPerPixel <= 0) throw new RangeError("pointsPerPixel must be positive");
  return largestTriangleThreeBuckets(points, Math.max(2, Math.floor(pixelWidth * pointsPerPixel)));
}
