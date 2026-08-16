# Trust levels

The engine distinguishes only data that is passed to validating APIs from data
that is already accepted by those APIs. Bundle readers should be treated as
parsers for untrusted bytes: they check archive offsets, paths, entry methods,
CRCs, checksums, binary tags, finite numeric values, and World construction.

There is no principal model, authenticated caller, role, capability, or policy
engine. A successful parse says that bytes meet the format's structural and
semantic checks; it does not establish who supplied them, whether they are
authorized, or whether their scientific claims are trustworthy.
