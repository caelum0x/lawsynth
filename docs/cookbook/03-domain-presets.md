# Pick the right settings per domain

**Goal:** get a sensible result without learning every knob. A **preset** (CLI)
or **recipe** (SDK) seeds a bundle of discovery settings tuned for a family of
systems. Explicit flags/overrides always win.

## CLI presets

List them:

```bash
lawsynth presets
```

**Expected shape:**

```
Discovery presets (use with `discover --preset <name>`):

  physics (alias: mechanics)
    Oscillatory & mechanical systems (polynomial + trig features)
    tunes:  polynomial degree 3; trigonometric features on (sin/cos); sparse threshold 0.05
    suits:  pendulum, van-der-pol
  ecology
    Predator-prey & logistic interactions (quadratic cross terms)
    tunes:  polynomial degree 2 (bilinear x*y interactions); sparse threshold 0.02
    suits:  lotka-volterra
  epidemiology
    ...
  finance
    ...
  general
    ...

Explicit flags (e.g. --degree, --threshold) always override the preset.
```

Apply a preset with `--preset`, then override any single knob:

```bash
# ecology: quadratic library, low threshold to keep small interaction terms
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld \
  --preset ecology

# physics: cubic + trig — for oscillators/pendulums
lawsynth new pendulum --data pend.csv
lawsynth discover pend.csv --time time --state theta,omega --output pend.lsworld \
  --preset physics

# start from a preset, then override just the threshold
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld \
  --preset ecology --threshold 0.01
```

Available preset names (incl. aliases): `ecology`, `epidemiology`, `finance`,
`general`, `mechanics`, `physics`.

## SDK recipes

```python
import lawsynth

lawsynth.recipes.names()                       # ('mechanics', 'ecology', 'epidemiology', 'finance', 'general')
print(lawsynth.recipes.get("ecology").describe())
```

**Expected shape** of `.describe()`:

```
Recipe: ecology
  A quadratic library capturing pairwise species interactions ...
  Suited to: Lotka–Volterra predator–prey, competitive Lotka–Volterra, ...
  Discovery settings (differences from defaults):
    derivative_method = 'finite'
    polynomial_degree = 2
    solver = 'stlsq'
    threshold = 0.05
  Explicit overrides passed to discover() always win.
```

Apply a recipe on a study; layer overrides on top:

```python
study = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])

result = study.discover(recipe="ecology")               # curated defaults
result = study.discover(recipe="mechanics", threshold=0.02)  # override wins
```

`recipe` and an explicit `config=` are mutually exclusive (a recipe *is* a
starting config); `**overrides` always win over either.

## Domains at a glance

| Domain | CLI preset (`--preset`) | SDK recipe (`recipe=`) |
|--------|--------------------------|------------------------|
| Mechanics / oscillators | `physics` (alias `mechanics`): degree 3, **trig on**, threshold 0.05 | `mechanics` (alias `physics`): degree 3, threshold 0.05, `stlsq` |
| Ecology / predator-prey | `ecology`: degree 2, threshold 0.02 | `ecology`: degree 2, threshold 0.05, `stlsq` |
| Epidemiology / compartments | `epidemiology`: degree 2, threshold 0.02 | `epidemiology`: degree 2, threshold 0.01, `stlsq` |
| Finance / rates | `finance`: degree 3, **rational on**, **refine on** | `finance`: degree 2, threshold 0.02, **`sr3` solver** |
| Unknown | `general`: degree 2, threshold 0.05 | `general`: degree 2, threshold 0.05, `stlsq` |

> **Presets and recipes are tuned independently and do not match knob-for-knob.**
> Notably, the CLI `physics` preset turns **trigonometric features on**, but the
> SDK `mechanics` recipe does **not** — for a pendulum in the SDK, add the toggle:
>
> ```python
> study.discover(recipe="mechanics", include_trigonometric=True)
> ```
>
> Likewise the CLI `finance` preset uses `stlsq` + rational + refinement, while
> the SDK `finance` recipe uses the `sr3` solver. Pick the surface, read what it
> tunes (`lawsynth presets` / `recipes.get(name).describe()`), and override as
> needed.

## See also

- [Discover from a CSV](01-discover-from-csv.md) for the base loop.
- [Take your model elsewhere](07-export-model.md) once you're happy with the fit.
