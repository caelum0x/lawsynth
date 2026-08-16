"""Version-aware S3 object ingestion with bounded body reads."""

from __future__ import annotations

import csv
import io
import json
from collections.abc import Iterable, Sequence
from typing import Any
from urllib.parse import urlparse

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import (
    ConnectorConnectionError,
    DataValidationError,
    LimitExceededError,
    ResourceNotFoundError,
)


def parse_s3_uri(resource: str) -> tuple[str, str]:
    parsed = urlparse(resource)
    key = parsed.path.lstrip("/")
    if parsed.scheme != "s3" or not parsed.netloc or not key:
        raise DataValidationError("S3 resource must use s3://bucket/key")
    return parsed.netloc, key


class S3Connector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, snapshots=True, projections=True)

    def _connect(self) -> None:
        boto3 = dependency("boto3", extra="s3", connector="s3")
        arguments: dict[str, Any] = {}
        endpoint = self.config.options.get("endpoint_url")
        region = self.config.options.get("region_name")
        if endpoint:
            arguments["endpoint_url"] = str(endpoint)
        if region:
            arguments["region_name"] = str(region)
        access_key = self.credentials.get("s3_access_key_id")
        secret_key = self.credentials.get("s3_secret_access_key")
        session_token = self.credentials.get("s3_session_token")
        if access_key or secret_key:
            if not (access_key and secret_key):
                raise DataValidationError("both S3 access key and secret key are required")
            arguments["aws_access_key_id"] = access_key.reveal()
            arguments["aws_secret_access_key"] = secret_key.reveal()
        if session_token:
            arguments["aws_session_token"] = session_token.reveal()
        self._client = boto3.client("s3", **arguments)

    def _close(self) -> None:
        client = getattr(self, "_client", None)
        if client is not None and hasattr(client, "close"):
            client.close()

    def _get(self, request: ReadRequest) -> tuple[bytes, dict[str, Any]]:
        bucket, key = parse_s3_uri(request.resource)
        arguments: dict[str, Any] = {"Bucket": bucket, "Key": key}
        version = request.snapshot or request.options.get("version_id")
        if version:
            arguments["VersionId"] = str(version)
        try:
            response = self._client.get_object(**arguments)
            body = response["Body"]
            declared = response.get("ContentLength")
            if isinstance(declared, int) and declared > self.config.max_bytes:
                raise LimitExceededError("S3 object exceeds max_bytes")
            chunks: list[bytes] = []
            total = 0
            while chunk := body.read(min(64 * 1024, self.config.max_bytes + 1)):
                total += len(chunk)
                if total > self.config.max_bytes:
                    raise LimitExceededError("S3 object exceeds max_bytes")
                chunks.append(chunk)
            return b"".join(chunks), response
        except LimitExceededError:
            raise
        except self._client.exceptions.NoSuchKey as exc:
            raise ResourceNotFoundError(
                f"S3 object does not exist: {request.resource}"
            ) from exc
        except Exception as exc:
            raise ConnectorConnectionError(
                "S3 object could not be read",
                connector=self.config.name,
                retryable=True,
                details={"exception_type": type(exc).__name__},
            ) from exc

    @staticmethod
    def _decode(payload: bytes, key: str, content_type: str | None) -> Sequence[Record]:
        try:
            text = payload.decode("utf-8-sig")
            if key.lower().endswith(".csv") or content_type == "text/csv":
                values: Any = list(csv.DictReader(io.StringIO(text)))
            elif key.lower().endswith((".jsonl", ".ndjson")):
                values = [json.loads(line) for line in text.splitlines() if line.strip()]
            else:
                values = json.loads(text)
        except (UnicodeDecodeError, csv.Error, json.JSONDecodeError) as exc:
            raise DataValidationError("S3 object is not valid JSON or CSV") from exc
        if isinstance(values, dict):
            values = [values]
        if not isinstance(values, list) or not all(isinstance(row, dict) for row in values):
            raise DataValidationError("S3 object must contain object records")
        return values

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        _bucket, key = parse_s3_uri(request.resource)
        payload, response = self._get(request)
        rows = self._decode(payload, key, response.get("ContentType"))
        stop = None if request.limit is None else request.offset + request.limit
        yield from rows[request.offset:stop]

    def _inspect(self, resource: str) -> ResourceInfo:
        bucket, key = parse_s3_uri(resource)
        try:
            response = self._client.head_object(Bucket=bucket, Key=key)
        except Exception as exc:
            raise ConnectorConnectionError(
                "S3 object metadata could not be read",
                connector=self.config.name,
                retryable=True,
            ) from exc
        return ResourceInfo(
            resource,
            True,
            byte_count=response.get("ContentLength"),
            snapshot=response.get("VersionId") or str(response.get("ETag", "")).strip('"'),
            metadata={
                "content_type": response.get("ContentType"),
                "last_modified": str(response.get("LastModified", "")),
                "storage_class": response.get("StorageClass"),
            },
        )
