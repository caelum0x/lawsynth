# Validation guards

Segmentation rejects empty inputs, non-finite values, invalid penalties, and insufficient observations. HMMs require every probability row to sum to one within `1e-9`. BOCPD rejects non-positive hazard, variance, and prior precision.

Validation prevents invalid numerical inputs; it does not establish that a selected regime model is appropriate for a system.
