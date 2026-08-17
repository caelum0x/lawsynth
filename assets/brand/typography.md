# LawSynth Typography

LawSynth is a scientific instrument, not a marketing site. The type system is
built for long reading sessions, dense tables, and equations — calm, editorial,
and legible before it is expressive.

## Type families

| Role | Family | Fallback stack | Used for |
| --- | --- | --- | --- |
| Display / wordmark | Georgia | `Georgia, "Times New Roman", serif` | The wordmark, page titles (`h1`), and section headings |
| Interface | Inter | `Inter, system-ui, sans-serif` | Body copy, navigation, buttons, form labels |
| Mono | UI monospace | `ui-monospace, SFMono-Regular, "SF Mono", monospace` | Code, equations, identifiers, kickers, and metadata |

The serif display face signals "published result"; the sans interface face keeps
controls quiet; the monospace face is reserved for anything the Rust core treats
as canonical (identifiers, units, equations, hashes).

## Type scale

A restrained modular scale (~1.25). Sizes are expressed in pixels to match the
values baked into the Studio and Playground stylesheets.

| Token | Size / line-height | Weight | Family | Usage |
| --- | --- | --- | --- | --- |
| `display` | 30 / 1.1 | 650 | Serif | Screen titles (`.lss-main h1`) |
| `title` | 19 / 1.0 | 700 | Serif | Wordmark, panel titles |
| `heading` | 15 / 1.3 | 650 | Serif | Card and section headings |
| `body` | 14 / 1.5 | 400 | Sans | Default reading text |
| `label` | 11 / 1.2 | 600 | Mono | Form labels, controls |
| `kicker` | 10–11 / 1.0 | 600–700 | Mono | Uppercase eyebrows, context strings |
| `code` | 13 / 1.55 | 400 | Mono | Editor, code blocks, equations |

## Weights

- Serif: **700** for the wordmark and titles, **650** for headings.
- Sans: **400** body, **600–650** for buttons and emphasized labels.
- Mono: **400** for code, **600** for labels and kickers.

Avoid faux-bold and italics in the interface face; reserve emphasis for weight
and the accent color (`#b54b2a`).

## Casing and tracking

- Kickers and labels are `UPPERCASE` with `0.08em`–`0.10em` letter-spacing.
- Headings and body use sentence case.
- Identifiers, units, and file names are never title-cased — render them verbatim
  in the monospace face (`prey`, `1/day`, `predator-prey.lsworld`).

## Accessibility

- Body text targets a minimum of 14px and never drops below 11px.
- Maintain WCAG AA contrast: `ink #18201d` on `paper #f3f0e8` and `surface #fffdf7`
  clears 4.5:1. Do not place `muted #59635e` text below 14px on `paper`.
- Line length for prose stays in the 60–75 character range.
- Respect `prefers-reduced-motion`; typography must never rely on animation to
  convey hierarchy.

See [`palette.json`](./palette.json) for the color tokens referenced above.
