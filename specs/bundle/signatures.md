# Authentication boundary

`BundleSignature::authenticate(secret, bytes)` computes an HMAC-SHA-256 tag encoded as lowercase hexadecimal, and `verify_signature` compares a supplied tag in constant time after checking length. It is a shared-secret MAC: whoever can verify can also create a tag.

This helper does not define a signed `.lsworld` layout. No signature entry is written or read, no key identifier or algorithm metadata is serialized, and no public-key signature, certificate, timestamp, or trust policy is implemented. Store the tag and secret-management metadata in a protocol outside the bundle, and authenticate the exact byte sequence transmitted.
