# Symbolic differentiation

`Expr::derivative(symbol)` returns the unsimplified symbolic derivative with respect to one identifier. It implements constant and symbol rules, linearity, product and quotient rules, and the general scalar power identity `u^v * (v' * log(u) + v * u'/u)`. Unary derivatives are `-u'`, `exp(u)u'`, `u'/u`, `cos(u)u'`, and `-sin(u)u'`.

The operation is syntactic. It may produce expressions with runtime domain restrictions not present in a special-case derivative (for example the general power rule contains `log(u)`), can grow rapidly, and does not simplify automatically. Apply `simplify` only with its documented local semantics; numerical or domain validation remains the caller's responsibility.
