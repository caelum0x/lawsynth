# Units

Variables and parameters can carry `lawsynth_units::Unit` values. With `WorldConfig.validate_units`, a world constructor infers the dimension of each law expression and compares it with the target state unit. A mismatch rejects construction.

Units protect dimensional consistency; they do not rescale input values. Convert data to a single chosen unit before populating a dataset or a simulation request.

Unit inference covers the expression operations implemented by the scalar IR. It does not provide arbitrary unit registries, offsets such as Celsius-to-Kelvin conversion, or dimensional analysis of external functions.
