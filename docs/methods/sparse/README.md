# Sparse regression

`lawsynth-sparse` fits a supplied numeric feature matrix to one numeric target. It includes ridge-backed sequential thresholded least squares (STLSQ), an SR3-style alternating relaxation, coordinate-descent lasso, nonnegative least squares, grouped thresholding, RMS scaling, and deterministic bootstrap selection frequencies.

These routines operate on already evaluated features. They do not construct expressions, select derivative methods, or attach statistical confidence claims to retained terms.
