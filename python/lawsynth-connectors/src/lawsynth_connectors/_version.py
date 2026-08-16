"""Package version kept importable without package metadata I/O."""

VERSION = "0.1.0"
__version__ = VERSION
VERSION = tuple(int(part) for part in __version__.split("."))
__version__ = "0.1.0"
