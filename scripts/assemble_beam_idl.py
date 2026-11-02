#!/usr/bin/env python3
"""Assemble a Beam IDL from Anchor 0.31 idl-build test output.

Anchor's IDL builder compiles the program with ``--features idl-build`` and
reads the ``__anchor_private_print_idl*`` tests. This script performs that
assembly for the checked-in file. It is not a deployment or a client.

The program has two ``#[error_code]`` enums. Anchor 0.31.1's collector keeps
only the last errors section, so this script concatenates every errors
section. It also rewrites ``BeamError`` codes from ``6000 + discriminant`` to
``7000 + discriminant``, because the printer ignores ``offset = 7000`` while
the generated ``From<BeamError> for u32`` honors it. Instructions, accounts,
events, and types are otherwise the compiler output.

Generate the log with the default feature set (``raydium-devnet``), from the
repository root. Leave ``ANCHOR_IDL_BUILD_RESOLUTION`` unset. Setting it makes
Anchor emit seed expressions that do not compile for this reconstructed
program, so the IDL omits PDA and address resolution.

    RUSTFLAGS="--cfg procmacro2_semver_exempt -A warnings" \\
    cargo test --manifest-path programs/beam/Cargo.toml --features idl-build \\
      --lib __anchor_private_print_idl -- --test-threads=1 --nocapture \\
      > /tmp/beam-idl/stdout.txt

Then:

    python3 scripts/assemble_beam_idl.py /tmp/beam-idl/stdout.txt sdk/idl/beam.json
"""
import json
import re
import sys
from pathlib import Path


def sections(text):
    """Split Anchor print-test stdout.

    With ``--nocapture`` the test harness joins ``test <name> ...`` onto the
    first printed line, so the begin marker is not always at column zero.
    """
    current = None
    buf = []
    found = []
    begin_re = re.compile(r"--- IDL begin ([a-z]+) ---")
    for line in text.splitlines():
        if current is None:
            match = begin_re.search(line)
            if match:
                current = match.group(1)
                rest = line[match.end() :]
                buf = [rest] if rest else []
            continue
        end = f"--- IDL end {current} ---"
        if end in line:
            prefix, _, _suffix = line.partition(end)
            if prefix:
                buf.append(prefix)
            found.append((current, "\n".join(buf)))
            current = None
            buf = []
            continue
        buf.append(line)
    if current is not None:
        raise SystemExit(f"unclosed IDL section {current}")
    return found


def convert_module_paths(idl):
    """Match anchor_lang_idl::build::convert_module_paths."""
    encoded = json.dumps(idl)
    pattern = re.compile(r'"(\w+::)+(\w+)"')
    # Apply sequentially, as the Rust fold does, so later names see earlier
    # replacements when checking for conflicts.
    cursor = 0
    out = []
    for match in pattern.finditer(encoded):
        out.append(encoded[cursor : match.start()])
        path = match.group(0)
        name = match.group(2)
        replaced = encoded.replace(path, f'"{name}"')
        if re.search(rf'"(\w+::)+{re.escape(name)}"', replaced):
            out.append(path)
        else:
            out.append(f'"{name}"')
        cursor = match.end()
    out.append(encoded[cursor:])
    return json.loads("".join(out))


def sort_idl(idl):
    for key in ("accounts", "constants", "events", "instructions", "types"):
        idl[key] = sorted(idl.get(key, []), key=lambda item: item["name"])
    idl["errors"] = sorted(idl.get("errors", []), key=lambda item: (item["code"], item["name"]))
    return idl


def verify(idl):
    name = idl["metadata"]["name"]
    codes = [err["code"] for err in idl["errors"]]
    if len(codes) != len(set(codes)):
        raise SystemExit(f"duplicate error codes in {name}")
    for account in idl["accounts"]:
        if "::" in account["name"]:
            raise SystemExit(f"conflicting account name in {name}: {account['name']}")
    for field in ("accounts", "events", "instructions"):
        for item in idl[field]:
            if not item["discriminator"]:
                raise SystemExit(f"empty {field} discriminator: {item['name']}")
        items = idl[field]
        for outer in items:
            for inner in items:
                if inner["name"] == outer["name"]:
                    continue
                disc = outer["discriminator"]
                other = inner["discriminator"]
                if disc[: len(other)] == other:
                    raise SystemExit(
                        f"ambiguous {field} discriminators: {outer['name']} and {inner['name']}"
                    )
    for account in idl["accounts"]:
        if all(byte == 0 for byte in account["discriminator"]):
            raise SystemExit(f"all-zero account discriminator: {account['name']}")
        zero_count = 0
        for byte in account["discriminator"]:
            if byte != 0:
                break
            zero_count += 1
        for other in idl["accounts"]:
            if len(other["discriminator"]) <= zero_count:
                raise SystemExit(
                    "accounts may allow substitution with the zero constraint: "
                    f"{account['name']} {other['name']}"
                )


def assemble(text):
    address = ""
    constants = []
    events = []
    errors = []
    types = {}
    program = None
    for kind, body in sections(text):
        if kind == "address":
            address = "".join(ch for ch in body if ch.isalnum())
        elif kind == "const":
            constants.append(json.loads(body))
        elif kind == "event":
            parsed = json.loads(body)
            events.append(parsed["event"])
            for ty in parsed["types"]:
                types[ty["name"]] = ty
        elif kind == "errors":
            errors.extend(json.loads(body))
        elif kind == "program":
            program = json.loads(body)
        else:
            raise SystemExit(f"unknown IDL section {kind}")
    if program is None:
        raise SystemExit("IDL program section missing")
    for ty in program.get("types", []):
        types[ty["name"]] = ty
    # Anchor 0.31's printer emits `6000 + discriminant` and ignores
    # `#[error_code(offset = N)]`. BeamError's runtime `From<u32>` adds the
    # declared offset 7000, so correct those seven codes before publication.
    beam_errors = {
        "MayhemDisabled",
        "PumpSwapDisabled",
        "MigrationNotEnabled",
        "InvalidRaydiumAccount",
        "InvalidMigrationBalance",
        "UnsupportedMint",
        "InvalidMigrationResult",
    }
    for err in errors:
        if err["name"] in beam_errors:
            err["code"] = err["code"] - 6000 + 7000
            beam_errors.remove(err["name"])
    if beam_errors:
        raise SystemExit(f"BeamError variants missing from IDL log: {sorted(beam_errors)}")
    program["address"] = address
    program["constants"] = constants
    program["events"] = events
    program["errors"] = errors
    program["types"] = list(types.values())
    program = convert_module_paths(program)
    program = sort_idl(program)
    verify(program)
    ordered = {}
    for key in (
        "address",
        "metadata",
        "docs",
        "instructions",
        "accounts",
        "events",
        "errors",
        "types",
        "constants",
    ):
        if key in program:
            ordered[key] = program[key]
    return ordered


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: assemble_beam_idl.py STDOUT_LOG OUTPUT_JSON")
    text = Path(sys.argv[1]).read_text(errors="replace")
    idl = assemble(text)
    destination = Path(sys.argv[2])
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(idl, indent=2) + "\n")
    print(
        f"wrote {destination}: {len(idl['instructions'])} instructions, "
        f"{len(idl['accounts'])} accounts, {len(idl['events'])} events, "
        f"{len(idl['errors'])} errors, {len(idl['types'])} types"
    )


if __name__ == "__main__":
    main()
