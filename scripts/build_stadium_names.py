#!/usr/bin/env python3
"""Build ``shared/stadium-names.csv`` from the installed FM27 database.

FM27 saves store a stadium name inline only for a couple of hundred grounds.
The game takes every other ground's name from its installed database, so this
script reads the first language section of ``lang_db.dat`` and writes the
stadium table as a two-column ``database_id,name`` CSV.

The table is a run of records::

    u32 uid | u32 name_len | name | u32 alt_name_len | alt_name | ff ff 00

The three-byte tail distinguishes the first language section's stadium table
from the club, city and competition tables around it.  The script stops after a
fixed decompressed prefix; the first language section's grounds all sit inside
it, and later sections use a different tail.

The ``zstd`` command-line tool is required to decompress ``lang_db.dat``.

Usage::

    python3 scripts/build_stadium_names.py
    python3 scripts/build_stadium_names.py --database-version 2700
    python3 scripts/build_stadium_names.py --lang-db /path/to/lang_db.dat
"""

from __future__ import annotations

import argparse
import csv
import re
import struct
import subprocess
import threading
from pathlib import Path

DEFAULT_DB_DIR = (
    Path.home()
    / "Library/Application Support/Steam/steamapps/common/Football Manager 27"
    / "fm.app/Contents/PlugIns/game_plugin.bundle/Contents/Resources"
    / "shared/data/database/db"
)
DEFAULT_OUTPUT = Path(__file__).resolve().parent.parent / "shared/stadium-names.csv"
ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
# The first language section's grounds end around 11 MB on the databases seen.
# Later sections repeat the names with a different record tail and are excluded.
DECOMPRESSED_PREFIX_BYTES = 24 * 1024 * 1024
STADIUM_TAIL = b"\xff\xff\x00"
MAX_NAME_BYTES = 160
MAX_UID = 300_000_000
TEXT_RUN = re.compile(rb"[^\x00-\x1f\x7f]{1,%d}" % MAX_NAME_BYTES)


def locate_lang_db(database_version: str | None) -> Path:
    if database_version is not None:
        if not database_version.isdigit():
            raise ValueError("--database-version must be numeric")
        return (
            DEFAULT_DB_DIR
            / database_version
            / f"{database_version}_fm"
            / "lang_db.dat"
        )
    versions = sorted(
        (
            entry
            for entry in DEFAULT_DB_DIR.iterdir()
            if entry.is_dir() and entry.name.isdigit()
        ),
        key=lambda entry: int(entry.name),
        reverse=True,
    )
    for version in versions:
        candidate = version / f"{version.name}_fm" / "lang_db.dat"
        if candidate.is_file():
            return candidate
    raise FileNotFoundError(f"no lang_db.dat under {DEFAULT_DB_DIR}")


def decompress_prefix(path: Path, limit: int) -> bytes:
    """Return the first ``limit`` decompressed bytes after the container header."""
    with path.open("rb") as handle:
        header = handle.read(256)
        magic = header.find(ZSTD_MAGIC)
        if magic < 0:
            raise ValueError(f"{path} has no zstd frame in its header")
        handle.seek(magic)
        process = subprocess.Popen(
            ["zstd", "-d", "-c"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        assert process.stdin is not None
        assert process.stdout is not None

        def feed() -> None:
            try:
                while chunk := handle.read(1024 * 1024):
                    process.stdin.write(chunk)
            except (BrokenPipeError, ValueError):
                pass
            finally:
                try:
                    process.stdin.close()
                except (BrokenPipeError, ValueError):
                    pass

        writer = threading.Thread(target=feed, daemon=True)
        writer.start()
        chunks: list[bytes] = []
        remaining = limit
        try:
            while remaining > 0:
                chunk = process.stdout.read(min(remaining, 1024 * 1024))
                if not chunk:
                    break
                chunks.append(chunk)
                remaining -= len(chunk)
        finally:
            process.kill()
            process.wait()
            writer.join(timeout=1)
        return b"".join(chunks)


def printable_text(data: bytes) -> str | None:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return None
    if not text.strip() or any(
        ord(character) < 0x20 or 0x7F <= ord(character) <= 0x9F for character in text
    ):
        return None
    return text


def parse_stadium_names(data: bytes) -> dict[int, str]:
    names: dict[int, str] = {}
    for match in TEXT_RUN.finditer(data):
        fresh_start = match.start()
        if fresh_start < 8:
            continue
        uid = struct.unpack_from("<I", data, fresh_start - 8)[0]
        name_length = struct.unpack_from("<I", data, fresh_start - 4)[0]
        name = printable_text(match.group())
        if name is None or name_length != len(match.group()):
            continue
        cursor = fresh_start + len(match.group())
        if cursor + 7 > len(data):
            continue
        alt_length = struct.unpack_from("<I", data, cursor)[0]
        if alt_length > MAX_NAME_BYTES:
            continue
        cursor += 4
        if alt_length:
            alt = data[cursor : cursor + alt_length]
            if len(alt) != alt_length or printable_text(alt) is None:
                continue
            cursor += alt_length
        if data[cursor : cursor + 3] != STADIUM_TAIL:
            continue
        next_uid = struct.unpack_from("<I", data, cursor + 3)[0]
        if not uid < next_uid <= MAX_UID:
            continue
        trimmed = name.strip()
        if trimmed:
            names.setdefault(uid, trimmed)
    return names


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lang-db", type=Path)
    parser.add_argument("--database-version")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    lang_db = args.lang_db or locate_lang_db(args.database_version)
    data = decompress_prefix(lang_db, DECOMPRESSED_PREFIX_BYTES)
    names = parse_stadium_names(data)
    if not 30_000 <= len(names) <= 60_000:
        raise SystemExit(
            f"found {len(names)} stadium names in {lang_db}; the layout may have changed"
        )
    for uid, expected in [
        (680, "Old Trafford"),
        (676, "Anfield"),
        (5114336, "Emirates Stadium"),
    ]:
        if names.get(uid) != expected:
            raise SystemExit(
                f"uid {uid} is {names.get(uid)!r}, expected {expected!r}"
            )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(["database_id", "name"])
        for uid in sorted(names):
            writer.writerow([uid, names[uid]])
    print(f"wrote {len(names)} stadium names to {args.output} from {lang_db}")


if __name__ == "__main__":
    main()
