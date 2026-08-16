import type { Rect, Size } from "./layout.js";
export interface TextMetrics { readonly width: number; readonly height: number; readonly lines: readonly string[]; }
export interface TextMeasureOptions { readonly fontSize?: number; readonly lineHeight?: number; readonly maxWidth?: number; readonly letterSpacing?: number; }
/** Deterministic estimate for pre-layout. It deliberately does not call canvas or the DOM. */
export function measureText(text: string, options: TextMeasureOptions = {}): TextMetrics {
  const fontSize = options.fontSize ?? 14, lineHeight = options.lineHeight ?? Math.round(fontSize * 1.3), letterSpacing = options.letterSpacing ?? 0, maxWidth = options.maxWidth ?? Infinity;
  if (fontSize <= 0 || lineHeight <= 0 || letterSpacing < 0 || maxWidth <= 0) throw new RangeError("invalid text measurement options");
  const charWidth = fontSize * 0.58 + letterSpacing, words = text.trim().length ? text.trim().split(/\s+/) : [""]; const lines: string[] = []; let current = "";
  for (const word of words) { const candidate = current ? `${current} ${word}` : word; if (current && candidate.length * charWidth > maxWidth) { lines.push(current); current = word; } else current = candidate; }
  lines.push(current); return { width: Math.min(maxWidth, Math.max(...lines.map((line) => line.length * charWidth))), height: lines.length * lineHeight, lines };
}
export function paddedSize(size: Size, padding: number | readonly [number, number, number, number]): Size { const p = typeof padding === "number" ? [padding, padding, padding, padding] as const : padding; if (p.some((value) => value < 0 || !Number.isFinite(value))) throw new RangeError("padding must be finite and non-negative"); return { width: size.width + p[1] + p[3], height: size.height + p[0] + p[2] }; }
export function insetRect(rect: Rect, padding: number): Rect { if (padding < 0 || padding * 2 > rect.width || padding * 2 > rect.height) throw new RangeError("padding cannot invert a rectangle"); return { x: rect.x + padding, y: rect.y + padding, width: rect.width - 2 * padding, height: rect.height - 2 * padding }; }
