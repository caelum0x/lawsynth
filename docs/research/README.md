# Research notes

This directory records how to make a LawSynth result inspectable and repeatable. It describes the methods that are implemented in the current Rust workspace: validated numeric datasets, deterministic differentiation and feature construction, sparse or bounded symbolic candidate search, deterministic scoring, World IR bundles, and numerical simulation.

These notes are not a claim that a selected equation is uniquely true or causal. A result is scientific evidence only when its data provenance, preprocessing choices, model class, held-out evaluation, and known failures are retained with it. Start with [methodology](methodology.md), then use [reproducibility](reproducibility.md) when preparing a result for another person to rerun.
