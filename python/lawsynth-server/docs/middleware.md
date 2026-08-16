# Middleware

`middleware.invoke` establishes a request ID and converts typed domain errors
to the public error envelope. Unexpected exceptions produce only a generic
500 response. Application code must raise `ServerError` subclasses for errors
that are safe to communicate to callers.
