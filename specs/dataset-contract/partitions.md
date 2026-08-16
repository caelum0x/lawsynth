# Partitions, batches, and windows

`Dataset::batches(batch_size)` returns owned, aligned batches in chronological
row order. A batch contains its original half-open row range, timestamps, and
all columns. `batch_size` must be greater than zero; the final batch may be
shorter.

`Dataset::windows(WindowConfig { width, step })` returns only complete sliding
windows. Width and step must be positive, and width may not exceed the dataset
length. Windows are emitted at starts `0, step, 2*step, ...` through the last
complete window.

These are in-memory views copied into owned vectors; they are not file
partitions and they do not change the source dataset fingerprint.
