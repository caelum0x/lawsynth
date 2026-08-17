"""Validate ``manifest.json`` and decode the ``world/world.bin`` payload.

The manifest is a byte-for-byte fixed document (``specs/bundle/manifest.md``).
The world payload is the binary-v1 encoding described in
``specs/bundle/layout.md``:

* magic ``LSW1`` (continuous) or ``LSD1`` (discrete)
* ``u32le`` variable count, each variable
* ``u32le`` parameter count, each parameter
* ``u32le`` law count, each law

A string is ``u16le`` length plus UTF-8 bytes.  A variable is a string id, one
role byte, then an optional unit.  A parameter is a string id, a little-endian
``f64`` value, then an optional unit.  A law is a string target followed by a
preorder expression.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass

from archive import MANIFEST_ENTRY, WORLD_ENTRY, Archive, InvalidArchive

EXPECTED_MANIFEST = (
    b"{\n"
    b'  "format": "lawsynth-world",\n'
    b'  "format_version": "0.1.0",\n'
    b'  "world_encoding": "lawsynth-world-binary-v1"\n'
    b"}\n"
)

ROLES = ("state", "control", "exogenous", "observed", "latent", "derived")
UNARY_OPS = ("negate", "exp", "log", "sin", "cos")
BINARY_OPS = ("add", "subtract", "multiply", "divide", "power")


@dataclass(frozen=True)
class Variable:
    id: str
    role: str
    unit: str | None


@dataclass(frozen=True)
class Parameter:
    id: str
    value: float
    unit: str | None


@dataclass(frozen=True)
class Law:
    target: str
    expression: str


@dataclass(frozen=True)
class World:
    kind: str
    variables: tuple[Variable, ...]
    parameters: tuple[Parameter, ...]
    laws: tuple[Law, ...]

    @property
    def state_count(self) -> int:
        return sum(1 for variable in self.variables if variable.role == "state")


def validate_manifest(archive: Archive) -> None:
    """Raise :class:`InvalidArchive` unless the manifest is byte-identical."""
    if archive.entry(MANIFEST_ENTRY) != EXPECTED_MANIFEST:
        raise InvalidArchive("unsupported manifest")


class _Cursor:
    __slots__ = ("data", "offset")

    def __init__(self, data: bytes) -> None:
        self.data = data
        self.offset = 0

    def take(self, count: int) -> bytes:
        end = self.offset + count
        if end > len(self.data):
            raise InvalidArchive("world payload ended prematurely")
        chunk = self.data[self.offset : end]
        self.offset = end
        return chunk

    def u16(self) -> int:
        return struct.unpack("<H", self.take(2))[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.take(4))[0]

    def f64(self) -> float:
        return struct.unpack("<d", self.take(8))[0]

    def byte(self) -> int:
        return self.take(1)[0]

    def string(self) -> str:
        length = self.u16()
        return self.take(length).decode("utf-8")

    def optional_string(self) -> str | None:
        present = self.byte()
        if present == 0:
            return None
        if present != 1:
            raise InvalidArchive("optional-string tag must be 0 or 1")
        return self.string()

    def at_end(self) -> bool:
        return self.offset == len(self.data)


def _decode_expression(cursor: _Cursor) -> str:
    tag = cursor.byte()
    if tag == 0:
        return _format_float(cursor.f64())
    if tag == 1:
        return cursor.string()
    if tag == 2:
        op = cursor.byte()
        if op >= len(UNARY_OPS):
            raise InvalidArchive(f"unknown unary operator {op}")
        return f"{UNARY_OPS[op]}({_decode_expression(cursor)})"
    if tag == 3:
        op = cursor.byte()
        if op >= len(BINARY_OPS):
            raise InvalidArchive(f"unknown binary operator {op}")
        left = _decode_expression(cursor)
        right = _decode_expression(cursor)
        return f"{BINARY_OPS[op]}({left}, {right})"
    raise InvalidArchive(f"unknown expression tag {tag}")


def _format_float(value: float) -> str:
    return f"{value:.12g}"


def decode_world(archive: Archive) -> World:
    """Decode ``world/world.bin`` into a structured :class:`World`."""
    cursor = _Cursor(archive.entry(WORLD_ENTRY))
    magic = cursor.take(4)
    if magic == b"LSW1":
        kind = "continuous"
    elif magic == b"LSD1":
        kind = "discrete"
    else:
        raise InvalidArchive(f"unknown world magic: {magic!r}")

    variables: list[Variable] = []
    for _ in range(cursor.u32()):
        identifier = cursor.string()
        role_byte = cursor.byte()
        if role_byte >= len(ROLES):
            raise InvalidArchive(f"unknown variable role {role_byte}")
        variables.append(Variable(identifier, ROLES[role_byte], cursor.optional_string()))

    parameters: list[Parameter] = []
    for _ in range(cursor.u32()):
        identifier = cursor.string()
        value = cursor.f64()
        parameters.append(Parameter(identifier, value, cursor.optional_string()))

    laws: list[Law] = []
    for _ in range(cursor.u32()):
        target = cursor.string()
        laws.append(Law(target, _decode_expression(cursor)))

    if not cursor.at_end():
        raise InvalidArchive("trailing bytes after final law")

    return World(
        kind=kind,
        variables=tuple(variables),
        parameters=tuple(parameters),
        laws=tuple(laws),
    )
