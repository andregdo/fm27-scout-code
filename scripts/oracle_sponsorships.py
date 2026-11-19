#!/usr/bin/env python3
"""Export club sponsorship contracts from the fmsave oracle.

This is a development tool. It is not shipped with the app. The oracle supports
the reference FM27 builds its checkout covers; use a save it can open (for
example the 27.0.0 reference save) when checking the Rust parser.

Usage:
    .venv/bin/python scripts/oracle_sponsorships.py \
        "/path/to/save.fm" --output /tmp/oracle_sponsorships.json
"""

from __future__ import annotations

import argparse
import json
import warnings
from datetime import date
from pathlib import Path

import fmsave


def sponsorship_row(row) -> dict:
    kind = row.kind
    raw_kind = getattr(kind, "raw", getattr(kind, "value", kind))
    return {
        "club_uid": row.club_uid,
        "kind": int(raw_kind),
        "start": row.start.isoformat() if isinstance(row.start, date) else None,
        "end": row.end.isoformat() if isinstance(row.end, date) else None,
        "total_value": row.total_value,
        "annual_value": row.annual_value,
    }


def open_save(path: str):
    try:
        return fmsave.open(path, experimental_fm27=True)
    except TypeError:
        # Newer fmsave releases detect FM27 without the preview flag.
        return fmsave.open(path)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("save")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    warnings.filterwarnings("ignore")
    save = open_save(args.save)
    rows = [sponsorship_row(row) for row in save.sponsorships()]
    payload = {
        "game_date": save.info.game_date.isoformat() if save.info.game_date else None,
        "sponsorships": rows,
    }
    Path(args.output).write_text(json.dumps(payload), encoding="utf-8")
    print(f"sponsorships={len(rows)}")
    save.close()


if __name__ == "__main__":
    main()
