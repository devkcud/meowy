# Artifact schema fixtures

The [artifact contract](../reference/artifact-formats.md) defines these JSON Schema
2020-12 files and the semantic checks that accompany them. They are bundled local
resources; `urn:meowy:schema:...` references never require network resolution.

For documentation validation, with Python `jsonschema` and `referencing` installed:

```sh
python3 docs/schemas/validate.py
```

This checks schema syntax, every example, source/member/distribution digests,
package IDs, canonical argument encoding, and rejection of invalid operation
fields or success-looking incomplete artifacts. It does not compile source,
execute a capsule, or certify the illustrative distribution as installable.
The examples deliberately include unavailable external payloads and mark their
capsule/report incomplete. `output.bin` contains seven bytes, `fixture`.
