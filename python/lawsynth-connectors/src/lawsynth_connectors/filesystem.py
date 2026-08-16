"""Sandboxed local filesystem connector for tabular interchange formats."""

from __future__ import annotations

import csv
import json
import os
import tempfile
from collections.abc import Iterable, Iterator, Mapping, Sequence
from pathlib import Path
from typing import Any, TextIO

from .base import (
    BaseConnector,
    ConnectorCapabilities,
    ReadRequest,
    Record,
    ResourceInfo,
    WriteRequest,
)
from .errors import (
    ConfigurationError,
    DataValidationError,
    LimitExceededError,
    ResourceNotFoundError,
)
from .fingerprints import fingerprint_file


class FilesystemConnector(BaseConnector):
    """Read and atomically write files below one configured root directory."""

    capabilities = ConnectorCapabilities(
        read=True,
        write=True,
        snapshots=True,
        projections=True,
    )

    def _connect(self) -> None:
        configured = self.config.option("root", str, default=".")
        self._root = Path(configured).expanduser().resolve()
        if not self._root.is_dir():
            raise ResourceNotFoundError(
                f"filesystem root does not exist: {self._root}",
                connector=self.config.name,
            )

    def _resolve(self, resource: str, *, must_exist: bool = True) -> Path:
        relative = Path(resource)
        if relative.is_absolute():
            raise ConfigurationError(
                "filesystem resources must be relative to the configured root",
                connector=self.config.name,
            )
        path = (self._root / relative).resolve(strict=False)
        if path != self._root and self._root not in path.parents:
            raise ConfigurationError(
                f"filesystem resource escapes configured root: {resource!r}",
                connector=self.config.name,
            )
        if must_exist and not path.is_file():
            raise ResourceNotFoundError(
                f"filesystem resource does not exist: {resource}",
                connector=self.config.name,
            )
        return path

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        path = self._resolve(request.resource)
        byte_count = path.stat().st_size
        if byte_count > self.config.max_bytes:
            raise LimitExceededError(
                "filesystem resource exceeds max_bytes",
                connector=self.config.name,
                details={"byte_count": byte_count, "max_bytes": self.config.max_bytes},
            )

        format_name = str(request.options.get("format", path.suffix.lstrip("."))).lower()
        if format_name in {"csv", "tsv"}:
            delimiter = "\t" if format_name == "tsv" else ","
            yield from self._read_delimited(path, request, delimiter)
        elif format_name in {"jsonl", "ndjson"}:
            yield from self._read_json_lines(path, request)
        elif format_name == "json":
            yield from self._read_json(path, request)
        elif format_name in {"parquet", "arrow", "feather", "ipc"}:
            from .arrow import records_from_arrow

            if format_name == "parquet":
                try:
                    import pyarrow.parquet as parquet
                except ImportError:
                    from .arrow import _arrow

                    _arrow()
                    raise
                source: Any = parquet.read_table(path, columns=list(request.columns) or None)
            else:
                source = path
            yield from records_from_arrow(
                source,
                columns=() if format_name == "parquet" else request.columns,
                offset=request.offset,
                limit=request.limit,
            )
        else:
            raise ConfigurationError(
                f"unsupported filesystem format: {format_name!r}",
                connector=self.config.name,
            )

    def _read_delimited(
        self,
        path: Path,
        request: ReadRequest,
        delimiter: str,
    ) -> Iterator[Record]:
        encoding = self._encoding(request)
        with path.open("r", encoding=encoding, newline="") as stream:
            reader = csv.DictReader(stream, delimiter=delimiter)
            if reader.fieldnames is None:
                raise DataValidationError("delimited resource has no header")
            missing = sorted(set(request.columns) - set(reader.fieldnames))
            if missing:
                raise DataValidationError(f"projection columns do not exist: {missing}")
            emitted = 0
            for index, row in enumerate(reader):
                if index < request.offset:
                    continue
                selected = request.columns or tuple(row)
                yield {column: row.get(column) for column in selected}
                emitted += 1
                if request.limit is not None and emitted >= request.limit:
                    break

    def _read_json_lines(self, path: Path, request: ReadRequest) -> Iterator[Record]:
        with path.open("r", encoding=self._encoding(request)) as stream:
            emitted = 0
            for line_number, line in enumerate(stream, start=1):
                if not line.strip():
                    continue
                try:
                    value = json.loads(line)
                except json.JSONDecodeError as exc:
                    raise DataValidationError(
                        f"invalid JSON on line {line_number}",
                        connector=self.config.name,
                    ) from exc
                if not isinstance(value, dict):
                    raise DataValidationError(
                        f"JSON line {line_number} is not an object",
                        connector=self.config.name,
                    )
                if line_number - 1 < request.offset:
                    continue
                yield self._project(value, request.columns)
                emitted += 1
                if request.limit is not None and emitted >= request.limit:
                    break

    def _read_json(self, path: Path, request: ReadRequest) -> Iterator[Record]:
        with path.open("r", encoding=self._encoding(request)) as stream:
            value = json.load(stream)
        if isinstance(value, dict):
            records = value.get("records")
            if records is None:
                records = [value]
        else:
            records = value
        if not isinstance(records, list):
            raise DataValidationError("JSON resource must contain an object array")

        stop = None if request.limit is None else request.offset + request.limit
        for record in records[request.offset:stop]:
            if not isinstance(record, dict):
                raise DataValidationError("JSON dataset contains a non-object record")
            yield self._project(record, request.columns)

    @staticmethod
    def _project(record: Mapping[str, Any], columns: Sequence[str]) -> Record:
        if not columns:
            return record
        missing = sorted(set(columns) - set(record))
        if missing:
            raise DataValidationError(f"projection columns do not exist: {missing}")
        return {column: record[column] for column in columns}

    @staticmethod
    def _encoding(request: ReadRequest) -> str:
        encoding = request.options.get("encoding", "utf-8")
        if not isinstance(encoding, str):
            raise ConfigurationError("filesystem encoding must be a string")
        return encoding

    def _write_records(
        self,
        request: WriteRequest,
        records: Sequence[Record],
        *,
        first_batch: bool,
    ) -> None:
        path = self._resolve(request.resource, must_exist=False)
        path.parent.mkdir(parents=True, exist_ok=True)
        format_name = str(request.options.get("format", path.suffix.lstrip("."))).lower()

        if format_name not in {"csv", "tsv", "jsonl", "ndjson"}:
            raise ConfigurationError(
                "filesystem writes support CSV, TSV, and JSON Lines",
                connector=self.config.name,
            )
        if first_batch and path.exists() and request.mode == "error":
            raise ConfigurationError(f"filesystem resource already exists: {request.resource}")

        if request.mode == "append" or not first_batch:
            self._append(path, format_name, records)
        else:
            self._atomic_replace(path, format_name, records)

    def _append(self, path: Path, format_name: str, records: Sequence[Record]) -> None:
        existing = path.exists() and path.stat().st_size > 0
        with path.open("a", encoding="utf-8", newline="") as stream:
            self._serialize(stream, format_name, records, write_header=not existing)

    def _atomic_replace(
        self,
        path: Path,
        format_name: str,
        records: Sequence[Record],
    ) -> None:
        descriptor, temporary_name = tempfile.mkstemp(
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            text=True,
        )
        try:
            with os.fdopen(descriptor, "w", encoding="utf-8", newline="") as stream:
                self._serialize(stream, format_name, records, write_header=True)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temporary_name, path)
        except BaseException:
            Path(temporary_name).unlink(missing_ok=True)
            raise

    @staticmethod
    def _serialize(
        stream: TextIO,
        format_name: str,
        records: Sequence[Record],
        *,
        write_header: bool,
    ) -> None:
        if format_name in {"jsonl", "ndjson"}:
            for record in records:
                stream.write(json.dumps(dict(record), ensure_ascii=False, default=str))
                stream.write("\n")
            return

        if not records:
            return
        fields = list(records[0])
        if any(set(record) != set(fields) for record in records):
            raise DataValidationError("CSV records must have consistent fields")
        writer = csv.DictWriter(
            stream,
            fieldnames=fields,
            delimiter="\t" if format_name == "tsv" else ",",
        )
        if write_header:
            writer.writeheader()
        writer.writerows(records)

    def _inspect(self, resource: str) -> ResourceInfo:
        path = self._resolve(resource, must_exist=False)
        if not path.is_file():
            return ResourceInfo(resource=resource, exists=False)
        stat = path.stat()
        fingerprint = fingerprint_file(path)
        return ResourceInfo(
            resource=resource,
            exists=True,
            byte_count=stat.st_size,
            snapshot=fingerprint.digest,
            metadata={"modified_ns": stat.st_mtime_ns, "suffix": path.suffix.lower()},
        )
