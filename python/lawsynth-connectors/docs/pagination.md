# Pagination

`lawsynth_connectors.pagination` supplies the bounded paging helpers and the
opaque, optionally signed cursors shared by remote connectors.

## Bounded chunks

`chunked(values, size)` yields immutable tuples of at most `size` items without
materializing the whole source. `BaseConnector` uses it to turn any record
iterable into `DataBatch` values sized by `config.batch_size`, so a connector
never holds an unbounded result set in memory.

## Pages and requests

- `PageRequest(size=100, cursor=None)` — `size` is validated to `1..10_000`.
- `Page(items, next_cursor, total)` — `has_more` is true when `next_cursor` is
  set.
- `paginate_sequence(values, request, codec=...)` — returns a stable offset page
  over an already materialized sequence and emits a cursor for the next page (or
  `None` at the end).

## Opaque cursors

`CursorCodec` encodes cursor state as URL-safe base64 with an optional HMAC-SHA-256
signature:

```python
from lawsynth_connectors.pagination import CursorCodec

codec = CursorCodec(secret=b"at-least-16-bytes-long")
token = codec.encode({"offset": 100})
state = codec.decode(token)  # {"offset": 100}
```

A signing secret shorter than 16 bytes is rejected. When a secret is configured,
`decode` verifies the signature in constant time and raises `DataValidationError`
on a tampered, truncated, or malformed cursor; it also rejects a payload that is
not a JSON object. Without a secret the cursor is still opaque but unauthenticated,
suitable only for trusted, in-process paging. Clients must treat the cursor as
opaque and never construct or parse it themselves.
