import { ShortcutRegistry } from "../src/shortcuts.js";

/** A controller-only shortcut setup that remains testable without a DOM. */
export function createNavigationShortcuts(onHome: () => void, onSettings: () => void): ShortcutRegistry {
  const shortcuts = new ShortcutRegistry();
  shortcuts.register({ id: "home", keys: "meta+shift+h", label: "Go home", scope: "global", run: onHome });
  shortcuts.register({ id: "settings", keys: "meta+,", label: "Open settings", scope: "global", run: onSettings });
  return shortcuts;
}
