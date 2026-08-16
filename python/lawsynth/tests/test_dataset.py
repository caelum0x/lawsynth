from lawsynth.dataset import Dataset
from lawsynth.errors import ValidationError


def test_dataset_validates_alignment_and_returns_native_owned_values():
    dataset = Dataset.from_columns([0, 1], {"x": [2, 3]})
    assert dataset.as_native_arguments() == ([0.0, 1.0], {"x": [2.0, 3.0]})
    try:
        Dataset.from_columns([0, 0], {"x": [1, 2]})
    except ValidationError:
        pass
    else:
        raise AssertionError("non-increasing time was accepted")
