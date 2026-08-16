import { defaultTokens, type DesignTokens, validateDesignTokens } from "./tokens.js";

export interface Theme {
  readonly name: string;
  readonly tokens: DesignTokens;
  readonly cssCustomProperties: Readonly<Record<string, string>>;
}

function cloneTokens(tokens: DesignTokens): DesignTokens {
  return Object.freeze({
    color: Object.freeze({ ...tokens.color }),
    space: Object.freeze({ ...tokens.space }),
    radius: Object.freeze({ ...tokens.radius }),
    typography: Object.freeze(Object.fromEntries(Object.entries(tokens.typography).map(([name, value]) => [name, Object.freeze({ ...value })])) as DesignTokens["typography"]),
    interaction: Object.freeze({ ...tokens.interaction }),
  });
}

/** Converts validated tokens into CSS values without deciding where or how they are rendered. */
export function cssCustomProperties(tokens: DesignTokens): Readonly<Record<string, string>> {
  validateDesignTokens(tokens);
  const properties: Record<string, string> = {};
  for (const [name, value] of Object.entries(tokens.color)) properties[`--ls-color-${name}`] = value;
  for (const [name, value] of Object.entries(tokens.space)) properties[`--ls-space-${name}`] = `${value}px`;
  for (const [name, value] of Object.entries(tokens.radius)) properties[`--ls-radius-${name}`] = `${value}px`;
  for (const [name, value] of Object.entries(tokens.typography)) {
    properties[`--ls-type-${name}-family`] = value.fontFamily;
    properties[`--ls-type-${name}-size`] = `${value.fontSizePx}px`;
    properties[`--ls-type-${name}-line-height`] = String(value.lineHeight);
    properties[`--ls-type-${name}-weight`] = String(value.fontWeight);
  }
  properties["--ls-focus-ring-width"] = `${tokens.interaction.focusRingWidthPx}px`;
  properties["--ls-focus-ring-offset"] = `${tokens.interaction.focusRingOffsetPx}px`;
  properties["--ls-target-min-size"] = `${tokens.interaction.targetMinSizePx}px`;
  properties["--ls-transition-ms"] = `${tokens.interaction.transitionMs}ms`;
  return Object.freeze(properties);
}

export function createTheme(name: string, tokens: DesignTokens = defaultTokens): Theme {
  if (!name.trim()) throw new RangeError("theme name cannot be blank");
  const copied = cloneTokens(tokens);
  validateDesignTokens(copied);
  return Object.freeze({ name, tokens: copied, cssCustomProperties: cssCustomProperties(copied) });
}

export const defaultTheme = createTheme("lawsynth", defaultTokens);
