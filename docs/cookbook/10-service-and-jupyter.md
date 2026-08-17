# Use LawSynth as a service, and explore in Jupyter

Two ways to drive the loop beyond a single CLI call: **discovery-as-a-service**
over HTTP, and **interactive exploration** in a notebook.

---

## Part A — LawSynth as a service (`Client` + `/v1`)

`lawsynth.Client` is a dependency-free (stdlib-only) client for a running
LawSynth API. It drives the *remote* loop: upload a dataset → submit a discovery
run → poll to completion → fetch/explain the world → forecast → write a report —
all over the service's `/v1` contract (bearer auth, `X-Api-Version: 1`, and the
`{"error": {code, message, request_id}}` envelope). Discovery runs on the
**server**; the client never touches the native engine.

### Connect

```python
import lawsynth

# against a live service
client = lawsynth.Client("http://localhost:8080", token="…")

print(client.version())     # {'version': ..., 'protocol': ...}  (GET /v1/version)
print(client.health())      # health probe                        (GET /v1/health)
```

### The run workflow

```python
run = client.submit_discovery(              # POST /v1/datasets, then POST /v1/runs
    csv="prey.csv", time="time", state=["x", "y"],
    preset="ecology", name="predator-prey",
)
run = client.wait(run)                      # poll GET /v1/runs/{id} until terminal
assert run.succeeded

world = client.world(run)                   # GET /v1/runs/{id}/world (→ /v1/worlds/{id})
print(client.explain(run.world_id))         # GET /v1/worlds/{id}/explain

fc = client.forecast(                       # POST /v1/worlds/{id}/forecast
    run.world_id,
    initial={"x": 10.0, "y": 5.0},
    horizon=4.0, step=0.1,
)
client.report(run.world_id, "world.html")   # GET /v1/worlds/{id}/report -> HTML file
```

`submit_discovery` references the dataset one of three ways (exactly one):
`dataset_id=`, inline `columns=`/`time=`, or a `csv=` path/literal. `preset=`
is resolved **client-side** through `lawsynth.recipes` into concrete discovery
knobs, so it works even against a service that only understands raw knobs;
`degree`/`threshold`/`solver`/feature toggles layer on top and win.

Other endpoints: `client.compare(left_id, right_id)` (POST `/v1/worlds/compare`),
`client.get_run(id)`, `client.get_world(id)`, `client.upload_dataset(...)`.

### Run it fully offline (in-process WSGI)

The same client drives the real API app object in-process — no socket, fully
deterministic — which is exactly how the shipped example
(`python/lawsynth/examples/service_client.py`) and the client tests run:

```python
from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_server.settings import Settings as ServerSettings

server = ServerSettings(
    database_url="sqlite:///metadata.sqlite3",
    object_root="objects",
    tokens={"0123456789abcdef0123456789abcdef": ("acme", frozenset({"read", "write"}))},
)
app = create_wsgi_app(ApiSettings(server=server, environment="test"))

client = lawsynth.Client(wsgi_app=app, token="0123456789abcdef0123456789abcdef")
# ... identical submit -> wait -> world -> explain -> forecast -> report loop
```

Run the shipped transcript with:

```bash
PYTHONPATH="python/lawsynth/src:services/api/src:python/lawsynth-server/src" \
  python3 python/lawsynth/examples/service_client.py
```

Errors surface as `lawsynth.ApiError` carrying `status`, `code`, and
`request_id`; an exhausted poll bound raises `lawsynth.RunTimeout`.

---

## Part B — Explore in Jupyter (`Study` dashboard + `explore()`)

The SDK objects render richly in Jupyter on their own, and the optional
`lawsynth-notebook` package adds a composed dashboard and a live interactive
widget.

### Rich auto-display

Just discover — `Study`, `DiscoveryResult`, `Explanation`, `Forecast`,
`ScenarioComparison`, `Ensemble`, and `MonitorReport` all define `_repr_html_`,
so returning one in a cell renders equations, charts, and tables:

```python
import lawsynth

study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
result = study.discover(recipe="ecology")
result                      # -> renders the study dashboard (or a compact report)
```

### The composed dashboard (`StudyDashboard`)

```python
dash = study.dashboard()            # requires the lawsynth-notebook package
dash                                # renders inline

# share it as a standalone HTML document
open("dashboard.html", "w").write(dash.to_document())
```

`Study.dashboard()` folds any registered scenarios into the view automatically.
You can also build one directly:

```python
from lawsynth_notebook import render_dashboard, StudyDashboard
dash = render_dashboard(result, theme="light")   # -> StudyDashboard
```

### The interactive explorer (`explore()`)

`explore()` turns a discovered world into a live `WorldExplorerWidget` (sliders
over initial conditions and time bounds, re-integrated on the fly):

```python
from lawsynth_notebook import explore, enable_explore

widget = explore(result)            # or explore(study) after discovery

# or attach an .explore() method onto the SDK/native classes:
enable_explore()
study.discover(recipe="ecology").explore()
```

`explore(source, *, initial=, start=, end=, step=, method="rk4", theme="light",
name=)` opens on the same baseline trajectory the SDK would produce, using the
object's own `equations`/`states`/`simulate`.

## See also

- [Discover from a CSV](01-discover-from-csv.md) for the local equivalent.
- [Organize & share your work](08-organize-and-share.md) — the service `/v1/projects` workspace.
