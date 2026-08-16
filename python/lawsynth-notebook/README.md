# LawSynth notebook support

`lawsynth-notebook` is a dependency-free presentation layer for validated,
portable LawSynth artifacts. It renders World metadata, equations, dependency
graphs, trajectories, candidate frontiers, events, regimes, and uncertainty as
safe HTML or ordinary JSON. It deliberately does not start a Jupyter server,
inject browser JavaScript, or execute model code from a bundle.

The package accepts already-decoded JSON mappings and trusted local files. For
native `.lsworld` archives, use `lawsynth.bundle.load` first, then pass the
resulting inspected metadata to `render_bundle`.

```python
from lawsynth_notebook import render_trajectory

view = render_trajectory({"time": [0, 1], "values": {"x": [1, 2]}})
display(view)                 # in IPython/Jupyter, via its HTML protocol
html = view.html              # elsewhere
```
