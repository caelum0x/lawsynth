"""Executable contract for SIR epidemic."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

import pytest

EXAMPLES = Path(__file__).resolve().parents[1]
if str(EXAMPLES) not in sys.path:
    sys.path.insert(0, str(EXAMPLES))
from _workflow import generate_example, load_example, verify_example

REPO = EXAMPLES.parent


def test_04_sir_epidemic_workflow() -> None:
    verify_example(Path(__file__).parent)


def _lawsynth_binary() -> Path | None:
    """Locate a built `lawsynth` CLI, preferring an on-PATH install."""
    found = shutil.which("lawsynth")
    if found:
        return Path(found)
    for profile in ("release", "debug"):
        candidate = REPO / "target" / profile / "lawsynth"
        if candidate.exists():
            return candidate
    return None


def _infected_rate_law(binary: Path, observations: Path) -> str:
    """Discover the SIR world at a physically meaningful threshold and return
    the recovered `dinfected/dt` law as printed by `explain`."""
    world = observations.parent / "sir-population-scale.lsworld"
    subprocess.run(
        [
            str(binary), "discover", str(observations),
            "--time", "time",
            "--state", "susceptible,infected,recovered",
            "--degree", "2", "--threshold", "0.02",
            "--output", str(world),
        ],
        cwd=REPO, check=True, capture_output=True, text=True,
    )
    explain = subprocess.run(
        [str(binary), "explain", str(world)],
        cwd=REPO, check=True, capture_output=True, text=True,
    )
    for line in explain.stdout.splitlines():
        if line.strip().startswith("dinfected/dt ="):
            return line.strip()
    raise AssertionError("explain did not emit a dinfected/dt law")


def test_04_sir_recovers_infection_coupling_at_population_scale() -> None:
    """Scale-invariance guard for the discovery engine.

    SIR states run to ~O(10^3), so the true bilinear infection coefficient is
    beta/N ~ 3.2e-4. Sparsity must be judged on each term's standardized
    contribution, not its raw coefficient magnitude, or the essential
    susceptible*infected coupling is silently pruned at any sane threshold and
    the epidemic law degenerates to a constant (the pre-fix failure was
    `dinfected/dt = 0.0915`). This drives the real engine and asserts the
    coupling survives at threshold 0.02. Skipped when the CLI has not been
    built (the pure-Python contract above still runs).
    """
    if load_example(Path(__file__).parent).config["kind"] != "sir":
        raise AssertionError("example kind changed away from SIR")

    binary = _lawsynth_binary()
    if binary is None:
        pytest.skip("lawsynth CLI not built; run `cargo build -p lawsynth-cli`")

    observations = generate_example(Path(__file__).parent)
    law = _infected_rate_law(binary, observations)

    # The susceptible*infected product must survive sparsity (either factor
    # order), with a coefficient near the true beta/N = 3.2e-4.
    coupling = re.search(
        r"(-?\d*\.?\d+(?:e-?\d+)?)\s*\*\s*"
        r"(?:infected\s*\*\s*susceptible|susceptible\s*\*\s*infected)",
        law,
    )
    assert coupling is not None, f"infection coupling S*I was pruned: {law}"
    value = float(coupling.group(1))
    assert 1e-4 <= value <= 6e-4, f"S*I coefficient {value} off the true 3.2e-4: {law}"
