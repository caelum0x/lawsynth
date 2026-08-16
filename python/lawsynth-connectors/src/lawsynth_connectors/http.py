"""Bounded HTTP ingestion with host controls and secret-safe headers."""

from __future__ import annotations

import csv
import io
import ipaddress
import json
import socket
from collections.abc import Iterable, Mapping, Sequence
from email.message import Message
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import Request, urlopen

from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import (
    ConfigurationError,
    ConnectorConnectionError,
    DataValidationError,
    LimitExceededError,
)

_SENSITIVE_HEADERS = {"authorization", "cookie", "proxy-authorization", "x-api-key"}


class HttpConnector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, snapshots=True, projections=True)

    def _validate_url(self, resource: str) -> str:
        parsed = urlparse(resource)
        if parsed.scheme not in {"http", "https"} or not parsed.hostname:
            raise DataValidationError("HTTP resource must be an absolute HTTP(S) URL")
        if parsed.username or parsed.password:
            raise DataValidationError("HTTP URL must not contain credentials")

        allowed = self.config.options.get("allowed_hosts", ())
        if allowed and parsed.hostname not in set(map(str, allowed)):
            raise DataValidationError(f"HTTP host is not allowlisted: {parsed.hostname}")
        allow_private = self.config.options.get("allow_private_network", False)
        if not isinstance(allow_private, bool):
            raise ConfigurationError("allow_private_network must be boolean")
        if not allow_private:
            self._reject_private_addresses(parsed.hostname, parsed.port)
        return resource

    @staticmethod
    def _reject_private_addresses(hostname: str, port: int | None) -> None:
        try:
            addresses = {
                entry[4][0]
                for entry in socket.getaddrinfo(hostname, port or 443, type=socket.SOCK_STREAM)
            }
        except socket.gaierror as exc:
            raise ConnectorConnectionError(
                "HTTP host could not be resolved", connector="http", retryable=True
            ) from exc
        for address in addresses:
            ip = ipaddress.ip_address(address)
            if not ip.is_global:
                raise DataValidationError(
                    f"HTTP host resolves to a non-public address: {address}"
                )

    def _headers(self) -> dict[str, str]:
        headers = {"Accept": "application/json, application/x-ndjson, text/csv;q=0.9"}
        configured = self.config.options.get("headers", {})
        if not isinstance(configured, Mapping):
            raise ConfigurationError("HTTP headers option must be a mapping")
        if any(str(key).lower() in _SENSITIVE_HEADERS for key in configured):
            raise ConfigurationError(
                "sensitive HTTP headers must come from a credential provider"
            )
        headers.update({str(key): str(value) for key, value in configured.items()})
        bearer = self.credentials.get("http_bearer_token")
        if bearer:
            headers["Authorization"] = f"Bearer {bearer.reveal()}"
        return headers

    def _fetch(self, resource: str, *, method: str = "GET") -> tuple[bytes, Message, int]:
        url = self._validate_url(resource)
        request = Request(url, headers=self._headers(), method=method)
        try:
            with urlopen(request, timeout=self.config.timeout_seconds) as response:
                if method == "HEAD":
                    return b"", response.headers, response.status
                return self._read_limited(response), response.headers, response.status
        except HTTPError as exc:
            raise ConnectorConnectionError(
                "HTTP source returned an error",
                connector=self.config.name,
                retryable=exc.code in {408, 425, 429, 500, 502, 503, 504},
                details={"status": exc.code},
            ) from exc
        except (URLError, TimeoutError, OSError) as exc:
            raise ConnectorConnectionError(
                "HTTP source could not be reached",
                connector=self.config.name,
                retryable=True,
                details={"exception_type": type(exc).__name__},
            ) from exc

    def _read_limited(self, response: Any) -> bytes:
        declared = response.headers.get("Content-Length")
        if declared and declared.isdigit() and int(declared) > self.config.max_bytes:
            raise LimitExceededError(
                "HTTP response exceeds max_bytes", connector=self.config.name
            )
        chunks: list[bytes] = []
        total = 0
        while chunk := response.read(min(64 * 1024, self.config.max_bytes + 1)):
            total += len(chunk)
            if total > self.config.max_bytes:
                raise LimitExceededError(
                    "HTTP response exceeds max_bytes", connector=self.config.name
                )
            chunks.append(chunk)
        return b"".join(chunks)

    def _decode(
        self,
        payload: bytes,
        headers: Message,
        resource: str,
    ) -> Sequence[Record]:
        charset = headers.get_content_charset("utf-8")
        try:
            text = payload.decode(charset)
        except (LookupError, UnicodeDecodeError) as exc:
            raise DataValidationError("HTTP response encoding is invalid") from exc

        content_type = headers.get_content_type()
        suffix = urlparse(resource).path.lower()
        try:
            if content_type in {"text/csv", "text/tab-separated-values"} or suffix.endswith((".csv", ".tsv")):
                delimiter = "\t" if content_type == "text/tab-separated-values" or suffix.endswith(".tsv") else ","
                values: Any = list(csv.DictReader(io.StringIO(text), delimiter=delimiter))
            elif content_type in {"application/x-ndjson", "application/jsonl"} or suffix.endswith((".jsonl", ".ndjson")):
                values = [json.loads(line) for line in text.splitlines() if line.strip()]
            else:
                values = json.loads(text)
        except (csv.Error, json.JSONDecodeError) as exc:
            raise DataValidationError("HTTP response is not valid tabular data") from exc
        if isinstance(values, dict):
            envelope_key = self.config.options.get("records_key")
            values = values.get(envelope_key) if envelope_key else [values]
        if not isinstance(values, list) or not all(isinstance(row, dict) for row in values):
            raise DataValidationError("HTTP response must contain object records")
        return values

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        payload, headers, _status = self._fetch(request.resource)
        rows = self._decode(payload, headers, request.resource)
        stop = None if request.limit is None else request.offset + request.limit
        yield from rows[request.offset:stop]

    def _inspect(self, resource: str) -> ResourceInfo:
        _payload, headers, status = self._fetch(resource, method="HEAD")
        length = headers.get("Content-Length")
        return ResourceInfo(
            resource,
            exists=200 <= status < 400,
            byte_count=int(length) if length and length.isdigit() else None,
            snapshot=headers.get("ETag"),
            metadata={
                "content_type": headers.get_content_type(),
                "last_modified": headers.get("Last-Modified"),
                "status": status,
            },
        )
