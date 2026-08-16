import { assertContrast, createTheme, defaultTokens } from "../src/index.js";

const theme = createTheme("analysis", defaultTokens);
assertContrast(theme.tokens.color.accentText, theme.tokens.color.accent);
console.log(theme.cssCustomProperties["--ls-color-accent"]);
