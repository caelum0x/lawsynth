# @lawsynth/design-system

`@lawsynth/design-system` is the portable semantic layer shared by LawSynth
clients. It supplies validated design tokens and serializable component
contracts; it intentionally does **not** ship a React, Vue, web-component, or
CSS renderer.

## What it guarantees

- Hex color tokens are checked for WCAG contrast: text pairs at 4.5:1 and the
  focus indicator at 3:1.
- Typography, spacing, target-size, motion, and focus-ring values are validated
  before a theme can be constructed.
- Components describe real HTML/ARIA semantics, focus obligations, and command
  identifiers. A client adapter maps those command identifiers to application
  behavior.
- All public values are data-only, so they can be logged, inspected, snapshot
  tested, or rendered in different environments without executable callbacks.

## Usage

```ts
import { createButton, createTheme, defaultTokens } from "@lawsynth/design-system";

const theme = createTheme("studio", defaultTokens);
const save = createButton({ id: "save-world", label: "Save world", action: "world.save" });
```

`save` is a `ComponentNode` with button semantics and an `activate` event whose
action is `world.save`. This package does not attach event listeners. Renderers
must honor generated `role`, ARIA attributes, hidden states, and dialog focus
contracts; they should use `theme.cssCustomProperties` when CSS is appropriate.

## Boundary

The package is deliberately not a UI renderer, layout engine, icon rasterizer,
or state manager. It cannot open a dialog, trap focus, animate a toast, or
execute an action on its own. Those behaviors require a concrete platform
adapter, which must consume these contracts faithfully.

Run `npm test` in this directory to compile and execute the contract tests.
