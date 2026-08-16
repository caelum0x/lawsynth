# xarray conversion

LawSynth discovery currently accepts a one-dimensional time axis and aligned numeric series. An xarray object with spatial, ensemble, or scenario dimensions must be reduced to an explicitly chosen one-dimensional observation table before fitting.

Choose the coordinate that represents physical time, state the unit and epoch, and document every selection or aggregation across other dimensions. For example, taking a spatial mean changes the target system; it should not be an implicit `.values` conversion. After selecting the series, convert the time coordinate and each data variable to finite Python lists for `Dataset`, or write the table to CSV.

Multi-dimensional field discovery and automatic xarray ingestion are not implemented. Keep the xarray preprocessing script in version control and test it against a small fixture with known coordinates.
