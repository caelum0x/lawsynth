"""Shared helpers deliberately avoid requiring a built native extension."""

from lawsynth.dataset import Dataset


def sample_dataset() -> Dataset:
    return Dataset.from_columns([0.0, 1.0, 2.0], {"x": [1.0, 2.0, 3.0]})
