# Content types and supported payloads

The format carries exactly one World IR payload at `world/world.bin`. Its media type is application-specific rather than registered; applications should identify it as `application/vnd.lawsynth.world+zip` only by local agreement, not as a public IANA claim. `manifest.json` is UTF-8 JSON and `checksums.sha256` is UTF-8 text.

`read_world` accepts only the continuous `LSW1` magic; `read_discrete_world` accepts only `LSD1`. Asking for the wrong time semantics fails. There is no multi-world archive, dataset payload, trajectory payload, plugin, source-code attachment, event schedule, regime schedule, or MIME dispatch mechanism in 0.1.
