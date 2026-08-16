# pandas conversion

The Python SDK does not import pandas. Convert a DataFrame deliberately, so a pandas dtype change cannot alter model input unnoticed.

```python
from lawsynth import Dataset

frame = frame.sort_values("time")
selected = frame[["time", "x", "y"]]
if selected.isna().any().any():
    raise ValueError("resolve missing observations before discovery")
dataset = Dataset.from_columns(
    selected["time"].tolist(),
    {name: selected[name].tolist() for name in ("x", "y")},
)
```

Validate units, timestamp conversion, duplicate times, and finite values before calling `Dataset.from_columns`. The SDK repeats structural validation, but it cannot know whether a timezone, resampling method, or interpolation policy is scientifically appropriate. Store those choices with the discovery run.
