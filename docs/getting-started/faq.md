# Frequently asked questions

Short, direct answers to the questions people ask before trying LawSynth —
what equation discovery is, how it compares to SINDy and symbolic-regression
tools, and what it can and cannot do today.

## What is "equation discovery from time-series data"?

It is the process of recovering an explicit mathematical model — usually a
system of differential equations `dx/dt = f(x)` — directly from measured
data, instead of hand-deriving it from theory or fitting an opaque black-box
model. You supply a time column and one or more numeric state columns; a
discovery algorithm searches a library of candidate terms (polynomial,
trigonometric, rational, …) and returns the sparsest combination that fits
the observed dynamics. The output is a readable equation, not a set of
weights — see [core concepts](/docs/concepts/equations) and the
[sparse regression guide](/docs/guides/discovery/sparse).

## What is SINDy, and is LawSynth "like SINDy"?

SINDy (Sparse Identification of Nonlinear Dynamics) is the leading published
method for this problem: it builds a library of candidate terms and solves a
sparse regression (originally sequentially-thresholded least squares) to keep
only the few terms that explain the measured derivatives. LawSynth implements
the same family of sparse-regression methods — STLSQ and SR3 solvers, with
stability selection — as one of its discovery solvers (`lawsynth discover
--solver stlsq|sr3`), alongside an optional bounded symbolic-search stage.
In plain terms: SINDy is a method, PySINDy is the reference Python library
for it, and LawSynth is a separate, Rust-core, deterministic implementation
of the same underlying idea, delivered as a CLI/SDK/Studio product rather
than a library you import into a notebook. See the
[sparse regression methods](/docs/methods/sparse/stlsq) and the
[full competitive comparison](/docs/research/competitive-analysis) for a
line-by-line feature comparison against PySINDy, PySR, Operon, SciML,
gplearn, and AI-Feynman.

## Is there a free, open-source alternative to PySINDy or symbolic-regression tools?

Yes. LawSynth is open source under Apache-2.0 and runs entirely on your own
machine. The core engine is Rust with a Python SDK (`maturin`/`pyo3`) and a
TypeScript Studio UI. There is no account, no API key, and no network call
during discovery — `cargo` is configured offline and the CLI never sends
your data anywhere. See [installation](/docs/getting-started/installation)
and the [determinism guide](/docs/guide/determinism) for what "offline and
deterministic" means in practice.

## How is LawSynth different from PySR, Operon, or other symbolic-regression libraries?

Pure symbolic-regression tools (PySR, Operon, gplearn) search for a single
best-fit formula and are typically stochastic — rerunning the same search
can return a different expression. LawSynth's discovery is **deterministic**:
identical inputs and options reproduce the identical `.lsworld` bundle, bit
for bit, every time (`lawsynth discover` writes the bundle; `lawsynth
inspect` reads it back). LawSynth also targets dynamics specifically
(systems of `dx/dt = f(x)` you can simulate forward), returns a Pareto front
of candidates ranked by fit versus complexity instead of one opaque score,
and ships the full loop — discover, explain, simulate, intervene, compare,
and share a signed, portable bundle — rather than only a formula string. The
[competitive analysis](/docs/research/competitive-analysis) documents where
each tool still leads (for example, PySINDy's weak/integral-form solvers and
PySR's distributed search are capabilities LawSynth does not yet have).

## Can I discover a governing equation from a CSV without writing code?

Yes, from the command line: `lawsynth discover observations.csv --time t
--state x,y --output model.lsworld` reads a CSV with a header and a strictly
increasing numeric time column, and writes a portable `.lsworld` bundle
reporting the leading candidate's mean-squared error and complexity. No
Python, Rust, or notebook code is required for this step. CSV, TSV, and a
constrained numeric Parquet subset are accepted. See the
[CSV-to-model cookbook](/docs/cookbook/01-discover-from-csv) and the
[`discover` CLI reference](/docs/reference/cli/discover) for every option.

## What is a `.lsworld` bundle?

It is LawSynth's portable output format: a single file holding the
discovered law system, its parameters, provenance, and (optionally) a
self-contained HTML report. `lawsynth inspect` reads it back, `lawsynth
simulate` runs it forward, and `lawsynth intervene` applies scheduled
interventions to it — so the same artifact you get from discovery is what
you share, version, and re-run later. See
[Worlds](/docs/concepts/world-ir/world) and
[organize & share your work](/docs/cookbook/08-organize-and-share).

## What does LawSynth cost?

Nothing. It is free and open source (Apache-2.0); there is no paid tier,
account, or usage limit — you build and run the CLI/SDK locally from the
[GitHub repository](https://github.com/caelum0x/lawsynth).

## What can't LawSynth do (yet)?

Stated plainly, so you can decide if it fits before trying it: discovery
does not infer missing values, latent states, measurement-error models, or
automatic causal identification — preprocessing and scientific judgment
remain your responsibility. The native Parquet reader only accepts an
uncompressed, flat, PLAIN-encoded numeric subset (it rejects other encodings
outright rather than risk silently misreading them). A computed simulation
trajectory is a numerical result, not a stability proof or uncertainty
certificate. LawSynth also does not yet publish an SRBench benchmark result
or ship an sklearn-compatible estimator — both are open, tracked gaps, not
finished features. Full detail: [limitations](/docs/research/limitations)
and [failure cases](/docs/research/failure-cases).

## Where do I start?

Follow the [quickstart](/docs/getting-started/quickstart) for the full
discover → understand → use → share loop on a two-state example, or jump
straight to [installation](/docs/getting-started/installation) if you
already have a CSV in hand.
