# Your first World

There are two ways to get a `.lsworld` bundle to work with: discover one from
observations, or scaffold one from a built-in template.

## Discover from observations

Follow the [quickstart](quickstart.md) CSV route when the law should be inferred from
data:

```sh
lawsynth discover observations.csv --time time --state x,y --output world.lsworld
```

The generated World is a canonical, validated archive — not a generic JSON document.
The Rust bundle writer chooses the layout and validates the World before writing it.

## Scaffold from a template

To explore the loop without your own data, generate a known world (and, optionally,
sample observations from it):

```sh
lawsynth templates
lawsynth new lorenz --output lorenz.lsworld --data lorenz.csv --samples 2000
```

Built-in templates: `lorenz`, `lotka-volterra`, `pendulum`, `van-der-pol`, `sir`.

## Inspect before reuse

```sh
lawsynth inspect world.lsworld
```

The inspector distinguishes continuous and discrete Worlds and reports state,
variable, and parameter counts. A failed inspection means the artifact is not
accepted by either bundle reader — a file extension is not proof of validity.

## Where to go next

- `lawsynth explain world.lsworld` — read the laws and their assumptions.
- `lawsynth simulate` / `lawsynth forecast` — run it forward and ask what-if.
- `lawsynth validate world.lsworld --data obs.csv --holdout 0.2` — score it on held-out data.
- `lawsynth report world.lsworld --output report.html` — share a self-contained report.

For a manually constructed World, use the Rust `lawsynth-world` API or the built
Python native API. Keep source values finite, identifiers valid, units compatible,
and exactly one law assigned to each state.
