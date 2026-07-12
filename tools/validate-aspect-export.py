#!/usr/bin/env python3
"""Validate issue-75 production export invariants (CI guard)."""

import json
import sys
from pathlib import Path

root = Path(sys.argv[1] if len(sys.argv) > 1 else "site")
entries = json.loads((root / "entries.json").read_text())
by_id = {e["id"]: e for e in entries}

for entry in entries:
    partners = entry.get("aspect_partners")
    assert isinstance(partners, list), f"aspect_partners is not an array: {entry['id']}"
    if entry.get("aspect") is not None or partners:
        assert entry["official"] and entry["pos"] == "verb", (
            "aspect metadata outside official verb", entry["id"], entry["title"]
        )
    for partner in partners:
        target = by_id.get(partner["id"])
        assert target is not None, ("missing partner page", entry["id"], partner)
        assert any(p["id"] == entry["id"] for p in target["aspect_partners"]), (
            "non-reciprocal partner", entry["id"], partner["id"]
        )

api = root / "api"
meta = json.loads((api / "meta.json").read_text())
lemmas = json.loads((api / "lemmas.json").read_text())
pairs = json.loads((api / "aspect-pairs.json").read_text())
assert meta["schema_version"] == 3
assert pairs["schema_version"] == 3 and len(pairs["pairs"]) == 1440
for row in lemmas["lemmas"]:
    assert len(row) == 8, ("lemma tuple width", len(row), row[:2])
    assert isinstance(row[7], list), ("aspect_partners is not an array", row[:2])
    if row[6] is not None:
        assert row[1] == "verb" and row[2] in ("official", "official-only"), row
for pair in pairs["pairs"]:
    for side in ("imperfective", "perfective"):
        entry_id = pair[side]["entry_id"]
        assert entry_id in by_id, ("pair endpoint missing page", side, pair[side])

# `total_bytes` is payload bytes and intentionally excludes meta.json itself.
counted = [api / "lemmas.json", api / "agent-guide.md", api / "router-selftest.json",
           api / "aspect-pairs.json"]
counted.extend((api / "forms").glob("*.json"))
actual_bytes = sum(path.stat().st_size for path in counted)
assert meta["total_bytes"] == actual_bytes, (
    "api total_bytes mismatch", meta["total_bytes"], actual_bytes
)
print(
    f"aspect export valid: {len(entries)} entries, "
    f"{sum(len(e['aspect_partners']) for e in entries)} directed links, "
    f"{len(pairs['pairs'])} model pairs, {len(lemmas['lemmas'])} API lemmas"
)
