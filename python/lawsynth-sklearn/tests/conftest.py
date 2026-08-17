"""Shared fixtures: deterministic synthetic dynamical systems."""

from __future__ import annotations

import math

import pytest


@pytest.fixture
def oscillator():
    """Harmonic oscillator: x = cos t, v = -sin t  (x' = v, v' = -x)."""
    n, dt = 160, 0.05
    t = [i * dt for i in range(n)]
    X = [[math.cos(ti), -math.sin(ti)] for ti in t]  # columns [x, v]
    return X, t


@pytest.fixture
def damped():
    """A lightly damped linear system with two coupled states."""
    n, dt = 200, 0.03
    t = [i * dt for i in range(n)]
    X = []
    for ti in t:
        env = math.exp(-0.15 * ti)
        X.append([env * math.cos(2.0 * ti), -2.0 * env * math.sin(2.0 * ti)])
    return X, t
