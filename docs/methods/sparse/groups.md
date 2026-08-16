# Grouped selection

`group_stlsq` applies STLSQ and then evaluates configured groups by their coefficient norm, retaining or removing all members of a group together according to the group threshold. Groups are validated for index bounds and overlap rules in the crate API.

This is group thresholding layered on the current solver; it is not a convex group-lasso optimizer and does not supply hierarchical or overlapping-group regularization.
