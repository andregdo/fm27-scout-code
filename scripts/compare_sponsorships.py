#!/usr/bin/env python3
"""Compare Rust club sponsorship output with the fmsave oracle export.

The comparison uses the factual contract fields only. The save stores no
sponsor name, and the club name is denormalised by both readers, so it is not
part of the key.

Usage:
    .venv/bin/python scripts/compare_sponsorships.py \
        /tmp/oracle_sponsorships.json /tmp/rust_sponsorships.json
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path

FIELDS = ("club_uid", "kind", "start", "end", "total_value", "annual_value")


def normalized_annual(value):
    """The save stores 0 where it keeps no annual value; that is unknown, not zero."""
    return None if value == 0 else value


def rows(path: Path) -> list[dict]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(payload, dict):
        return payload.get("sponsorships", [])
    return payload


def key(row: dict) -> tuple:
    values = [row.get(field) for field in FIELDS]
    values[FIELDS.index("annual_value")] = normalized_annual(values[FIELDS.index("annual_value")])
    return tuple(values)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("oracle")
    parser.add_argument("rust")
    args = parser.parse_args()

    oracle = rows(Path(args.oracle))
    rust = rows(Path(args.rust))
    oracle_counts = Counter(key(row) for row in oracle)
    rust_counts = Counter(key(row) for row in rust)
    missing = oracle_counts - rust_counts
    extra = rust_counts - oracle_counts
    oracle_clubs = {row.get("club_uid") for row in oracle}
    extra_shared = Counter(
        {row: count for row, count in extra.items() if row[0] in oracle_clubs}
    )
    extra_coverage = extra - extra_shared

    print(
        f"sponsorships: oracle={len(oracle)} rust={len(rust)} "
        f"missing={sum(missing.values())} extra_shared={sum(extra_shared.values())} "
        f"extra_coverage={sum(extra_coverage.values())}"
    )
    print(
        f"coverage: oracle_clubs={len(oracle_clubs)} "
        f"rust_clubs={len({row.get('club_uid') for row in rust})} "
        f"extra_clubs={len({row[0] for row in extra_coverage})}"
    )
    for label, differences in (
        ("missing", missing),
        ("extra_shared", extra_shared),
        ("extra_coverage", extra_coverage),
    ):
        for row, count in list(differences.items())[:10]:
            values = dict(zip(FIELDS, row))
            print(f"  {label} x{count}: {values}")
    if missing or extra_shared:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
