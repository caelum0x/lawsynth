# Export

`export_html` writes a static document and `export_json` writes canonical
artifact data. `reproducible_notebook_cell` returns an ordinary nbformat cell
whose source recreates a static JSON view; it never embeds results or executes
a LawSynth world.
