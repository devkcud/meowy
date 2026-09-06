"""Validate documentation schemas and fixtures; does not run meowy or native code."""

import json
from copy import deepcopy
from hashlib import sha256
from pathlib import Path

from jsonschema import Draft202012Validator
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parent
EXAMPLES = ROOT / "examples"


def digest(data):
    return "sha256:" + sha256(data).hexdigest()


def canonical(value):
    """The profile's integer-only canonical JSON, including control escaping."""
    if value is None:
        return b"null"
    if value is True:
        return b"true"
    if value is False:
        return b"false"
    if isinstance(value, int):
        assert -(2**53 - 1) <= value <= 2**53 - 1
        return str(value).encode("ascii")
    if isinstance(value, str):
        parts = ['"']
        for char in value:
            code = ord(char)
            assert not 0xD800 <= code <= 0xDFFF
            if char in ('"', "\\"):
                parts.append("\\" + char)
            elif code < 32:
                parts.append(f"\\u{code:04x}")
            else:
                parts.append(char)
        return ("".join(parts) + '"').encode("utf-8")
    if isinstance(value, list):
        return b"[" + b",".join(canonical(item) for item in value) + b"]"
    if isinstance(value, dict):
        keys = sorted(value, key=lambda key: key.encode("utf-8"))
        return (
            b"{"
            + b",".join(canonical(key) + b":" + canonical(value[key]) for key in keys)
            + b"}"
        )
    raise AssertionError(f"Unsupported canonical value: {type(value)}")


schemas = {}
for path in sorted(ROOT.glob("*.schema.json")):
    schema = json.loads(path.read_text())
    Draft202012Validator.check_schema(schema)
    schemas[schema["$id"]] = schema
registry = Registry().with_resources(
    (identity, Resource.from_contents(schema)) for identity, schema in schemas.items()
)


def validator(value):
    identity = "urn:meowy:schema:" + value["schema"].removeprefix("meowy.") + ":1"
    return Draft202012Validator(schemas[identity], registry=registry)


documents = {}
for name in (
    "mod.lock",
    "diagnostic.json",
    "capsule.json",
    "build-report.json",
    "distribution.json",
    "runtime-event.json",
):
    value = json.loads((EXAMPLES / name).read_text())
    validator(value).validate(value)
    documents[name] = value

distribution_digest = digest((EXAMPLES / "distribution.json").read_bytes())
for name in ("mod.lock", "capsule.json", "build-report.json"):
    value = documents[name]
    identity = (
        value["inputs"]["distribution"]
        if name == "build-report.json"
        else value["distribution"]
    )
    assert identity["digest"] == distribution_digest

lock = documents["mod.lock"]
package_ids = set()
for package in lock["packages"]:
    key = "\0".join(package[field] for field in ("source", "revision", "directory"))
    assert package["id"] == "git:" + sha256(key.encode()).hexdigest()
    assert package["id"] not in package_ids
    package_ids.add(package["id"])
for request in lock["requests"]:
    assert request["package"] in package_ids

capsule = documents["capsule.json"]
assert capsule["capture"]["state"] == "incomplete" and capsule["missing"]
assert [item["path"] for item in capsule["inventory"]] == sorted(
    item["path"] for item in capsule["inventory"]
)
for item in capsule["inventory"]:
    data = (EXAMPLES / item["path"]).read_bytes()
    assert len(data) == item["bytes"] and digest(data) == item["digest"]
for source in capsule["inputs"]:
    data = (EXAMPLES / source["member"]).read_bytes()
    assert len(data) == source["bytes"] and digest(data) == source["digest"]
span = documents["diagnostic.json"]["primary"]
data = (EXAMPLES / "source.txt").read_bytes()
assert digest(data) == span["digest"] and 0 <= span["start"] <= span["end"] <= len(data)
assert data[span["start"] : span["end"]].decode() == 'debug.panic("fixture")'

event = documents["runtime-event.json"]
assert event["arguments_digest"] == digest(canonical(event["arguments"]))
payload = b""
for item in event["outcome"]["payloads"]:
    data = (EXAMPLES / item["member"]).read_bytes()
    assert len(data) == item["bytes"] and digest(data) == item["digest"]
    payload += data
assert len(payload) == int(event["outcome"]["data"]["count"])
assert event["arguments"]["bytes_digest"] == digest(payload)
assert (
    canonical({"z": "\n", "é": -1, "a": True})
    == b'{"a":true,"z":"\\u000a","\xc3\xa9":-1}'
)

# These mutations exercise protocol boundaries that an open dictionary would miss.
bad = deepcopy(event)
bad["arguments"]["arbitrary_key"] = 1
assert not validator(bad).is_valid(bad)
bad = deepcopy(event)
bad["operation"] = "task.join"
assert not validator(bad).is_valid(bad)
bad = deepcopy(event)
bad["outcome"]["data"]["unexpected"] = False
assert not validator(bad).is_valid(bad)
bad = deepcopy(documents["build-report.json"])
bad["status"] = "complete"
bad["failed_phase"] = None
assert not validator(bad).is_valid(bad)
bad = deepcopy(capsule)
bad["capture"] = {"state": "closed", "reasons": []}
assert not validator(bad).is_valid(bad)

print(
    f"Validated {len(schemas)} schemas, {len(documents)} examples, payload identities and rejection cases."
)
