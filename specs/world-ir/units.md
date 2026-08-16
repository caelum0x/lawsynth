# Units

Units are scaled SI dimensions with exponent order `[length, mass, time, current, temperature, amount, luminous intensity]`. The built-in parser accepts only `1`, `m`, `km`, `s`, `min`, `kg`, and `g`, combined left-to-right with `*`, `/`, and integer `^` exponents. It preserves the supplied expression as its canonical display string; it does not algebraically normalize equivalent spellings.

Addition and subtraction require equal dimensions. `exp`, `log`, `sin`, and `cos` require dimensionless operands. Multiplication and division combine dimensions. A non-dimensionless base raised to a power requires an integer constant exponent in the inclusive `i8` range; a dimensionless base remains dimensionless. Overflow of any signed `i8` dimension exponent is rejected.

Scale affects conversion but not expression-dimension equality. For example, `m/s` and `km/min` are dimension-compatible. Unit checking reports unknown symbols, incompatible dimensions, or dimension overflow rather than silently coercing values.
