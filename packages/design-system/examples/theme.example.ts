import { createTheme, defaultTokens } from "../src/index.js";

const darkTokens = { ...defaultTokens, color: { ...defaultTokens.color, canvas: "#0f172a", surface: "#1e293b", text: "#f8fafc", mutedText: "#cbd5e1", accent: "#60a5fa", accentText: "#172554", danger: "#fca5a5", dangerText: "#450a0a", border: "#64748b", focusRing: "#93c5fd" } };
console.log(createTheme("dark", darkTokens).cssCustomProperties["--ls-color-canvas"]);
