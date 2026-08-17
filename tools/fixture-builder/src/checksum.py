"""Content digests for fixtures.

Two digests are provided, matching ``specs/reproducibility/data-hash.md``:

* :func:`sha256_hex` — SHA-256 over the exact byte stream. Use this to identify
  or verify fixture content.
* :func:`stable_hash` — a 64-bit FNV-1a hash for deterministic local keys and
  seed derivation only. It is *not* collision resistant and must not be used to
  prove integrity of scientific input.
"""

from __future__ import annotations

import hashlib

_FNV_OFFSET_BASIS = 0xCBF29CE484222325
_FNV_PRIME = 0x100000001B3
_MASK64 = 0xFFFFFFFFFFFFFFFF


def sha256_hex(data: bytes) -> str:
    """Return the lowercase hex SHA-256 digest of ``data``."""
    return hashlib.sha256(data).hexdigest()


def stable_hash(data: bytes) -> int:
    """Return the 64-bit FNV-1a hash of ``data`` (for seeds and local keys)."""
    digest = _FNV_OFFSET_BASIS
    for byte in data:
        digest ^= byte
        digest = (digest * _FNV_PRIME) & _MASK64
    return digest


def seed_from(*parts: str) -> int:
    """Derive a deterministic 63-bit seed from string parts (for generators)."""
    joined = "\x1f".join(parts).encode("utf-8")
    return stable_hash(joined) & 0x7FFFFFFFFFFFFFFF
