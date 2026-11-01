#!/usr/bin/env python3
"""Verify the immutable archive and the explicit Beam source delta."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
manifest = json.loads((ROOT / "upstream/manifest.json").read_text())
snapshot = ROOT / "upstream/pumpfun-source"
files = json.loads((ROOT / "upstream/files.json").read_text())
def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
for path, expected in files.items():
    if git_blob((snapshot / path).read_bytes()) != expected["git_blob"]:
        raise SystemExit(f"Pristine upstream file changed: {path}")
for path, expected in manifest["raydium"]["files"].items():
    if git_blob((ROOT / "upstream/raydium-cp-swap" / path).read_bytes()) != expected:
        raise SystemExit(f"Pristine Raydium reference changed: {path}")
original, fork = snapshot / "programs/pump", ROOT / "programs/beam"
changed, unchanged = [], 0
original_paths = {str(p.relative_to(original)) for p in original.rglob("*") if p.is_file()}
for path in sorted(original_paths):
    p = fork / path
    if not p.is_file():
        raise SystemExit(f"Recovered source removed: {path}")
    if p.read_bytes() != (original / path).read_bytes():
        if path not in manifest["allowed_modified_paths"]:
            raise SystemExit(f"Unapproved retained-source modification: {path}")
        changed.append(path)
    else:
        unchanged += 1
for p in fork.rglob("*"):
    if p.is_file() and "target" not in p.parts:
        path = str(p.relative_to(fork))
        if path not in original_paths and path not in manifest["allowed_added_paths"]:
            raise SystemExit(f"Unapproved addition: {path}")
print(f"Pristine archive: {len(files)} Git blobs verified at {manifest['pumpfun']['commit']}")
print(f"Retained Pump package: {unchanged} files byte-identical; {len(changed)} explicitly modified")
print("Modified paths: " + ", ".join(changed))
print("Source provenance is not whole-program binary identity or runtime parity.")
