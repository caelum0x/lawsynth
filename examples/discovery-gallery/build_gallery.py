#!/usr/bin/env python3
"""Build the LawSynth discovery gallery: run discovery on several example
datasets, simulate the recovered world, and emit deterministic SVG plots plus a
Markdown showcase page for the docs site (lawsynth.dev).

The output is fully deterministic (no timestamps, no randomness), so the site's
byte-identical-render test keeps passing. Regenerate with:

    python3 examples/discovery-gallery/build_gallery.py

Artifacts:
  * apps/docs-site/assets/showcase-<name>.svg   (served at site root)
  * docs/showcase/discovery-gallery.md          (auto-published under /docs)
"""
from __future__ import annotations

import csv
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / "target" / "debug" / "lawsynth"
ASSETS = ROOT / "apps" / "docs-site" / "assets"
PAGE = ROOT / "docs" / "showcase" / "discovery-gallery.md"
WORK = ROOT / "examples" / "discovery-gallery" / "_work"

# Deterministic categorical palette (matches the site's dark theme accents).
COLORS = ["#6ea8fe", "#f7a072", "#8ce99a", "#e599f7", "#ffd43b"]

# Each entry: how to discover and simulate one dataset, plus the ground truth we
# are trying to recover. `flags` are appended to `discover`; `sim` drives the
# forward simulation from the recovered world.
DATASETS = [
    {
        "name": "lorenz",
        "title": "Lorenz attractor (chaotic, 3-state)",
        "obs": "examples/01-lorenz/output/observations.csv",
        "states": ["x", "y", "z"],
        "flags": ["--degree", "2", "--threshold", "1.0"],
        "sim": {"initial": {"x": 1.0, "y": 1.0, "z": 1.0}, "start": 0.0, "end": 4.0, "step": 0.01},
        "truth": [
            "dx/dt = 10 (y - x)",
            "dy/dt = 28 x - x z - y",
            "dz/dt = x y - 2.667 z",
        ],
        "blurb": "Recovers the two linear terms of the x-equation and the "
        "bilinear x·z / x·y couplings that make the system chaotic.",
    },
    {
        "name": "lotka-volterra",
        "title": "Lotka–Volterra predator–prey (2-state)",
        "obs": "examples/02-lotka-volterra/output/observations.csv",
        "states": ["prey", "predator"],
        "flags": ["--degree", "2", "--threshold", "0.05"],
        "sim": {"initial": {"prey": 10.0, "predator": 5.0}, "start": 0.0, "end": 12.0, "step": 0.05},
        "truth": [
            "dprey/dt = 1.1 prey - 0.4 prey·predator",
            "dpredator/dt = 0.1 prey·predator - 0.4 predator",
        ],
        "blurb": "Near-exact recovery of all four rate constants from a clean "
        "oscillatory trajectory.",
    },
    {
        "name": "sir",
        "title": "SIR epidemic (population-scale, 3-state)",
        "obs": "examples/04-sir-epidemic/output/observations.csv",
        "states": ["susceptible", "infected", "recovered"],
        "flags": ["--degree", "2", "--threshold", "0.02"],
        "sim": {
            "initial": {"susceptible": 990.0, "infected": 10.0, "recovered": 0.0},
            "start": 0.0,
            "end": 80.0,
            "step": 0.25,
        },
        "truth": [
            "dS/dt = -(β/N) S·I           (β/N ≈ 3.2e-4)",
            "dI/dt =  (β/N) S·I - γ I",
            "dR/dt =  γ I                 (γ = 0.1)",
        ],
        "blurb": "The scale-invariance fix in action: with states ~O(10³) the "
        "true interaction coefficient is ~3e-4, and the recovered S·I coupling "
        "now survives sparsity at a physically meaningful threshold.",
    },
    {
        "name": "damped-pendulum",
        "title": "Damped pendulum (nonlinear, 2-state)",
        "obs": "examples/03-damped-pendulum/output/observations.csv",
        "states": ["theta", "omega"],
        "flags": ["--degree", "2", "--trigonometric", "--threshold", "0.05"],
        "sim": {"initial": {"theta": 1.1, "omega": 0.0}, "start": 0.0, "end": 12.0, "step": 0.05},
        "truth": [
            "dθ/dt = ω",
            "dω/dt = -9.81 sin(θ) - 0.25 ω",
        ],
        "blurb": "With a trigonometric library, recovery captures the sin(θ) "
        "restoring force and the linear viscous damping.",
    },
]


def run(args: list[str]) -> str:
    result = subprocess.run(
        [str(BIN), *args], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return result.stdout


def read_csv(path: Path) -> tuple[list[str], list[list[float]]]:
    with path.open() as handle:
        rows = list(csv.reader(handle))
    header = rows[0]
    data = [[float(v) for v in row] for row in rows[1:] if row]
    return header, data


def discover_and_simulate(spec: dict) -> dict:
    WORK.mkdir(parents=True, exist_ok=True)
    world = WORK / f"{spec['name']}.lsworld"
    obs = ROOT / spec["obs"]
    run(
        [
            "discover", str(obs), "--time", "time",
            "--state", ",".join(spec["states"]),
            "--output", str(world), *spec["flags"],
        ]
    )
    laws = [
        line.strip()
        for line in run(["explain", str(world)]).splitlines()
        if "/dt =" in line
    ]
    sim = spec["sim"]
    sim_args = ["simulate", str(world), "--start", str(sim["start"]),
                "--end", str(sim["end"]), "--step", str(sim["step"])]
    for name, value in sim["initial"].items():
        sim_args += ["--initial", f"{name}={value}"]
    sim_csv = WORK / f"{spec['name']}-sim.csv"
    sim_csv.write_text(run(sim_args))
    return {"laws": laws, "sim_csv": sim_csv, "obs_csv": obs}


def _series(header: list[str], data: list[list[float]], column: str) -> tuple[list[float], list[float]]:
    idx = header.index(column)
    tcol = header.index("time")
    return [row[tcol] for row in data], [row[idx] for row in data]


def build_svg(spec: dict, artifacts: dict) -> str:
    obs_header, obs_data = read_csv(artifacts["obs_csv"])
    sim_header, sim_data = read_csv(artifacts["sim_csv"])

    width, height = 760, 380
    ml, mr, mt, mb = 54, 150, 20, 40
    plot_w, plot_h = width - ml - mr, height - mt - mb

    # Global bounds across observed + simulated for every plotted state.
    xs_all: list[float] = []
    ys_all: list[float] = []
    for state in spec["states"]:
        ot, ov = _series(obs_header, obs_data, state)
        st, sv = _series(sim_header, sim_data, state)
        xs_all += ot + st
        ys_all += ov + sv
    xmin, xmax = min(xs_all), max(xs_all)
    ymin, ymax = min(ys_all), max(ys_all)
    if xmax == xmin:
        xmax += 1.0
    if ymax == ymin:
        ymax += 1.0

    def px(x: float) -> float:
        return ml + (x - xmin) / (xmax - xmin) * plot_w

    def py(y: float) -> float:
        return mt + (1.0 - (y - ymin) / (ymax - ymin)) * plot_h

    def path(times: list[float], values: list[float]) -> str:
        pts = " ".join(f"{px(t):.2f},{py(v):.2f}" for t, v in zip(times, values))
        return pts

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" '
        f'role="img" aria-label="{spec["title"]}: observed vs. simulated from the recovered law" '
        f'font-family="ui-sans-serif,system-ui,sans-serif">',
        f'<rect width="{width}" height="{height}" fill="#0b1020"/>',
        # Plot frame.
        f'<rect x="{ml}" y="{mt}" width="{plot_w}" height="{plot_h}" fill="#0f1630" '
        f'stroke="#26304f"/>',
        f'<text x="{ml}" y="{height - 12}" fill="#8a94b0" font-size="12">time →</text>',
    ]
    legend_x = ml + plot_w + 16
    ly = mt + 6
    for i, state in enumerate(spec["states"]):
        color = COLORS[i % len(COLORS)]
        ot, ov = _series(obs_header, obs_data, state)
        st, sv = _series(sim_header, sim_data, state)
        # Observed: faint wide underlay. Simulated-from-law: crisp colored line.
        parts.append(
            f'<polyline fill="none" stroke="{color}" stroke-opacity="0.28" '
            f'stroke-width="6" points="{path(ot, ov)}"/>'
        )
        parts.append(
            f'<polyline fill="none" stroke="{color}" stroke-width="1.8" '
            f'points="{path(st, sv)}"/>'
        )
        parts.append(
            f'<rect x="{legend_x}" y="{ly - 9}" width="16" height="4" fill="{color}"/>'
        )
        parts.append(
            f'<text x="{legend_x + 22}" y="{ly - 4}" fill="#c9d2ea" font-size="12">{state}</text>'
        )
        ly += 22
    parts.append(
        f'<text x="{legend_x}" y="{ly + 8}" fill="#8a94b0" font-size="11">'
        f'thick = observed</text>'
    )
    parts.append(
        f'<text x="{legend_x}" y="{ly + 24}" fill="#8a94b0" font-size="11">'
        f'thin = recovered law</text>'
    )
    parts.append("</svg>")
    return "\n".join(parts)


def build_page(results: list[dict]) -> str:
    lines = [
        "# Discovery gallery",
        "",
        "Each panel below is produced end-to-end by the LawSynth CLI: observations",
        "→ `discover` (SINDy with RMS-standardized sparsity) → `simulate` the",
        "recovered world forward from its initial condition. The **thick faint**",
        "line is the observed trajectory; the **thin bright** line is the system",
        "*re-simulated from the equations LawSynth inferred*. Overlap means the",
        "recovered law reproduces the data.",
        "",
        "Regenerate with `python3 examples/discovery-gallery/build_gallery.py`.",
        "",
    ]
    for spec, artifacts in results:
        lines += [
            f"## {spec['title']}",
            "",
            f"![{spec['title']}: observed vs. simulated from the recovered law]"
            f"(/showcase-{spec['name']}.svg)",
            "",
            spec["blurb"],
            "",
            "**Recovered laws**",
            "",
            "```",
            *artifacts["laws"],
            "```",
            "",
            "**Ground truth**",
            "",
            "```",
            *spec["truth"],
            "```",
            "",
        ]
    return "\n".join(lines) + "\n"


def main() -> int:
    if not BIN.exists():
        print(f"build the CLI first: cargo build -p lawsynth-cli (missing {BIN})", file=sys.stderr)
        return 1
    ASSETS.mkdir(parents=True, exist_ok=True)
    PAGE.parent.mkdir(parents=True, exist_ok=True)
    results = []
    for spec in DATASETS:
        artifacts = discover_and_simulate(spec)
        svg = build_svg(spec, artifacts)
        (ASSETS / f"showcase-{spec['name']}.svg").write_text(svg)
        results.append((spec, artifacts))
        print(f"[{spec['name']}] {len(artifacts['laws'])} laws, svg written")
    PAGE.write_text(build_page(results))
    print(f"wrote {PAGE.relative_to(ROOT)} and {len(results)} SVG(s) under {ASSETS.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
