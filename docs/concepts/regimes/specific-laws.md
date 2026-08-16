# Regime-specific affine laws

`RegimeLawBook` stores one `AffineLaw { slope, intercept }` per finite regime index and evaluates it for a scalar input. It rejects a duplicate entry and reports an error for a missing regime law.

This type supports explicit, inspectable regime-indexed scalar behavior. It does not discover the laws, connect them to a World’s expression IR, or simulate switching continuous dynamics.
