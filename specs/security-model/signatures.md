# Authentication tags

`BundleSignature::authenticate(secret, bytes)` computes HMAC-SHA-256 and
`verify_signature` compares lowercase hexadecimal tags in constant time after
checking length. This authenticates exact bytes only to holders of the same
secret. It is not a public-key signature.

There is no signature entry in a `.lsworld` archive, no key id, key rotation,
certificate, timestamp, signer identity, revocation, signature policy, or CLI
verification workflow. Keep the tag, key reference, algorithm policy, and
authenticated transport in an external system. Bundle SHA-256 checksums alone
are integrity checks, not authentication.
