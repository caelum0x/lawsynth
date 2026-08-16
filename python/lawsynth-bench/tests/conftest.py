import pytest
from lawsynth_bench.dataset import Observation

@pytest.fixture
def rows():
    return [Observation("linear", "lawsynth", "wall_time", 4.0, "ms", "one"), Observation("linear", "lawsynth", "wall_time", 6.0, "ms", "two")]
