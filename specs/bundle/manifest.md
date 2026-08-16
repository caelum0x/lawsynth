# Manifest

The entry `manifest.json` must be byte-for-byte the writer’s fixed UTF-8 content:

```json
{
  "format": "lawsynth-world",
  "format_version": "0.1.0",
  "world_encoding": "lawsynth-world-binary-v1"
}
```

Readers do not implement a general JSON parser or tolerate alternate whitespace, additional keys, or semantically equivalent values: any byte difference is `InvalidArchive("unsupported manifest")`. The manifest identifies the archive contract, not an application run or author.
