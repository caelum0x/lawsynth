/** A primitive value that a renderer may safely project onto an element. */
export type AttributeValue = string | number | boolean;

/**
 * A serializable, framework-neutral component description. It deliberately
 * contains no DOM nodes, callbacks, or renderer state; adapters own those.
 */
export interface ComponentNode {
  readonly tag: string;
  readonly role?: string;
  readonly attributes: Readonly<Record<string, AttributeValue>>;
  readonly text?: string;
  readonly children: readonly ComponentNode[];
  readonly events: readonly ComponentEvent[];
  readonly focus?: FocusContract;
}

export interface ComponentEvent {
  readonly type: "activate" | "change" | "dismiss" | "submit" | "select";
  /** An application-defined command identifier; never executable code. */
  readonly action: string;
}

export interface FocusContract {
  readonly initialId?: string;
  readonly returnId?: string;
  readonly trap: boolean;
}

export function componentNode(
  tag: string,
  attributes: Readonly<Record<string, AttributeValue>> = {},
  options: Omit<ComponentNode, "tag" | "attributes" | "children" | "events"> & {
    readonly children?: readonly ComponentNode[];
    readonly events?: readonly ComponentEvent[];
  } = {},
): ComponentNode {
  if (!tag.trim()) throw new RangeError("component tag cannot be blank");
  for (const [name, value] of Object.entries(attributes)) {
    if (!name.trim() || value === "") throw new RangeError(`invalid component attribute: ${name}`);
  }
  return Object.freeze({
    tag,
    attributes: Object.freeze({ ...attributes }),
    children: Object.freeze([...(options.children ?? [])]),
    events: Object.freeze([...(options.events ?? [])]),
    ...(options.role === undefined ? {} : { role: options.role }),
    ...(options.text === undefined ? {} : { text: options.text }),
    ...(options.focus === undefined ? {} : { focus: Object.freeze({ ...options.focus }) }),
  });
}

export interface PaletteTokens {
  readonly canvas: string;
  readonly surface: string;
  readonly text: string;
  readonly mutedText: string;
  readonly accent: string;
  readonly accentText: string;
  readonly danger: string;
  readonly dangerText: string;
  readonly border: string;
  readonly focusRing: string;
}

export interface TypographyToken {
  readonly fontFamily: string;
  readonly fontSizePx: number;
  readonly lineHeight: number;
  readonly fontWeight: 400 | 500 | 600 | 700;
}

export interface TypographyTokens {
  readonly body: TypographyToken;
  readonly label: TypographyToken;
  readonly heading: TypographyToken;
  readonly monospace: TypographyToken;
}

export interface InteractionTokens {
  readonly focusRingWidthPx: number;
  readonly focusRingOffsetPx: number;
  readonly targetMinSizePx: number;
  readonly transitionMs: number;
}

export interface DesignTokens {
  readonly color: PaletteTokens;
  readonly space: Readonly<Record<"xs" | "sm" | "md" | "lg" | "xl", number>>;
  readonly radius: Readonly<Record<"sm" | "md" | "lg", number>>;
  readonly typography: TypographyTokens;
  readonly interaction: InteractionTokens;
}

interface Rgb { readonly red: number; readonly green: number; readonly blue: number; }

/** Parses CSS hexadecimal colors. Rejecting named/rgb colors keeps validation deterministic. */
export function parseHexColor(value: string): Rgb {
  const match = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(value);
  if (!match) throw new RangeError(`expected an opaque hexadecimal color, received ${value}`);
  const raw = match[1];
  if (raw === undefined) throw new RangeError("invalid color");
  const expanded = raw.length === 3 ? raw.split("").map((part) => part + part).join("") : raw;
  return Object.freeze({
    red: Number.parseInt(expanded.slice(0, 2), 16),
    green: Number.parseInt(expanded.slice(2, 4), 16),
    blue: Number.parseInt(expanded.slice(4, 6), 16),
  });
}

function linearChannel(channel: number): number {
  const normalized = channel / 255;
  return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
}

export function relativeLuminance(color: string): number {
  const rgb = parseHexColor(color);
  return 0.2126 * linearChannel(rgb.red) + 0.7152 * linearChannel(rgb.green) + 0.0722 * linearChannel(rgb.blue);
}

export function contrastRatio(foreground: string, background: string): number {
  const [lighter, darker] = [relativeLuminance(foreground), relativeLuminance(background)].sort((a, b) => b - a);
  if (lighter === undefined || darker === undefined) throw new RangeError("cannot calculate contrast");
  return (lighter + 0.05) / (darker + 0.05);
}

export function assertContrast(foreground: string, background: string, minimum = 4.5): void {
  if (!Number.isFinite(minimum) || minimum < 1) throw new RangeError("contrast minimum must be at least one");
  const actual = contrastRatio(foreground, background);
  if (actual < minimum) throw new RangeError(`insufficient color contrast: ${actual.toFixed(2)} < ${minimum}`);
}

function assertPositiveInteger(value: number, name: string): void {
  if (!Number.isInteger(value) || value <= 0) throw new RangeError(`${name} must be a positive integer`);
}

function assertTypography(token: TypographyToken, name: string): void {
  if (!token.fontFamily.trim()) throw new RangeError(`${name}.fontFamily cannot be blank`);
  if (!Number.isFinite(token.fontSizePx) || token.fontSizePx < 10) throw new RangeError(`${name}.fontSizePx must be at least 10`);
  if (!Number.isFinite(token.lineHeight) || token.lineHeight < 1) throw new RangeError(`${name}.lineHeight must be at least 1`);
}

/** Validates the WCAG-relevant invariants required by every LawSynth theme. */
export function validateDesignTokens(tokens: DesignTokens): void {
  for (const color of Object.values(tokens.color)) parseHexColor(color);
  assertContrast(tokens.color.text, tokens.color.canvas);
  assertContrast(tokens.color.mutedText, tokens.color.canvas);
  assertContrast(tokens.color.accentText, tokens.color.accent);
  assertContrast(tokens.color.dangerText, tokens.color.danger);
  assertContrast(tokens.color.focusRing, tokens.color.canvas, 3);
  for (const [name, value] of Object.entries(tokens.space)) assertPositiveInteger(value, `space.${name}`);
  if (!(tokens.space.xs < tokens.space.sm && tokens.space.sm < tokens.space.md && tokens.space.md < tokens.space.lg && tokens.space.lg < tokens.space.xl)) {
    throw new RangeError("spacing tokens must be strictly increasing");
  }
  for (const [name, value] of Object.entries(tokens.radius)) {
    if (!Number.isFinite(value) || value < 0) throw new RangeError(`radius.${name} must be non-negative`);
  }
  for (const [name, value] of Object.entries(tokens.typography)) assertTypography(value, `typography.${name}`);
  assertPositiveInteger(tokens.interaction.focusRingWidthPx, "interaction.focusRingWidthPx");
  if (!Number.isFinite(tokens.interaction.focusRingOffsetPx) || tokens.interaction.focusRingOffsetPx < 0) throw new RangeError("focus ring offset must be non-negative");
  if (tokens.interaction.targetMinSizePx < 24) throw new RangeError("interactive targets must be at least 24px");
  if (!Number.isFinite(tokens.interaction.transitionMs) || tokens.interaction.transitionMs < 0) throw new RangeError("transition duration must be non-negative");
}

export const defaultTokens: DesignTokens = Object.freeze({
  color: Object.freeze({ canvas: "#ffffff", surface: "#f8fafc", text: "#0f172a", mutedText: "#475569", accent: "#1d4ed8", accentText: "#ffffff", danger: "#b91c1c", dangerText: "#ffffff", border: "#94a3b8", focusRing: "#2563eb" }),
  space: Object.freeze({ xs: 4, sm: 8, md: 12, lg: 16, xl: 24 }),
  radius: Object.freeze({ sm: 4, md: 8, lg: 12 }),
  typography: Object.freeze({
    body: Object.freeze({ fontFamily: "Inter, system-ui, sans-serif", fontSizePx: 14, lineHeight: 1.5, fontWeight: 400 }),
    label: Object.freeze({ fontFamily: "Inter, system-ui, sans-serif", fontSizePx: 14, lineHeight: 1.3, fontWeight: 600 }),
    heading: Object.freeze({ fontFamily: "Inter, system-ui, sans-serif", fontSizePx: 20, lineHeight: 1.2, fontWeight: 700 }),
    monospace: Object.freeze({ fontFamily: "ui-monospace, SFMono-Regular, monospace", fontSizePx: 13, lineHeight: 1.5, fontWeight: 400 }),
  }),
  interaction: Object.freeze({ focusRingWidthPx: 2, focusRingOffsetPx: 2, targetMinSizePx: 32, transitionMs: 120 }),
});

validateDesignTokens(defaultTokens);
