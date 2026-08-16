# Pagination

PageRequest::new(offset, limit, maximum_limit) validates limit > 0 and limit <=
maximum_limit; Page<T> carries items, the accepted offset/limit, and an optional
total count. These are in-process value types, not query-string or cursor
definitions.

Offset pagination is vulnerable to concurrent collection changes. A service
using it MUST define a stable sort order and document snapshot/consistency
semantics. Cursor tokens, default limits, and response JSON are not specified.
