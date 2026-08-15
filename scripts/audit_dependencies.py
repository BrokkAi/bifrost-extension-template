#!/usr/bin/env python3
"""Create or verify the deterministic license inventory from Cargo metadata."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def render_inventory(metadata: dict[str, object]) -> str:
    workspace_members = set(metadata["workspace_members"])
    rows: list[tuple[str, str, str, str]] = []

    for package in metadata["packages"]:
        package_id = package["id"]
        license_expression = package.get("license")
        source = package.get("source")

        if not license_expression:
            raise ValueError(f"{package_id} has no declared license")
        if package_id in workspace_members:
            source_label = "workspace"
        elif source == REGISTRY_SOURCE:
            source_label = "crates.io"
        else:
            raise ValueError(f"{package_id} has disallowed dependency source: {source!r}")

        rows.append(
            (package["name"], package["version"], license_expression, source_label)
        )

    rows.sort(key=lambda row: (row[0].casefold(), row[1], row[3]))
    lines = ["name\tversion\tlicense\tsource"]
    lines.extend("\t".join(row) for row in rows)
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--check", type=Path, metavar="PATH")
    action.add_argument("--write", type=Path, metavar="PATH")
    args = parser.parse_args()

    try:
        inventory = render_inventory(json.load(sys.stdin))
    except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"dependency audit failed: {error}", file=sys.stderr)
        return 1

    if args.write:
        args.write.parent.mkdir(parents=True, exist_ok=True)
        args.write.write_text(inventory, encoding="utf-8")
        return 0

    expected = args.check.read_text(encoding="utf-8")
    if expected != inventory:
        print(
            "dependency inventory differs from Cargo.lock; regenerate it with:\n"
            "  cargo metadata --locked --format-version 1 | "
            "python3 scripts/audit_dependencies.py --write "
            "audits/dependency-licenses.tsv",
            file=sys.stderr,
        )
        return 1
    print(f"verified {len(inventory.splitlines()) - 1} dependency license records")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
