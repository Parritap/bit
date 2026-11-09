#!/usr/bin/env python3
"""
read_index.py - Read and display the contents of a Git index file.

Supports index versions 2, 3 and 4, decodes the TREE and REUC extensions,
lists any other extension by name and size, and verifies the checksum.
Only the Python standard library is used.

Usage:
    python3 read_index.py                    # reads .git/index, full detail
    python3 read_index.py path/to/index      # reads another file
    python3 read_index.py --short            # like `git ls-files --stage`
    python3 read_index.py --no-ext           # skip extension decoding
    python3 read_index.py --hash sha256      # for SHA-256 repositories
"""

import argparse
import hashlib
import struct
import sys
from datetime import datetime, timezone

OBJECT_TYPES = {0b1000: "regular file", 0b1010: "symlink", 0b1110: "gitlink"}


class IndexError_(Exception):
    """Raised when the index file is malformed."""


# --------------------------------------------------------------------------
# Low-level helpers
# --------------------------------------------------------------------------


def read_varint(buf, off):
    """Variable-width integer used by index v4 (same as OFS_DELTA in packs)."""
    byte = buf[off]
    off += 1
    value = byte & 0x7F
    while byte & 0x80:
        byte = buf[off]
        off += 1
        value = ((value + 1) << 7) | (byte & 0x7F)
    return value, off


def read_cstring(buf, off):
    """Read a NUL-terminated byte string. Returns (bytes, offset_after_NUL)."""
    end = buf.index(b"\x00", off)
    return buf[off:end], end + 1


def fmt_time(seconds, nanos):
    dt = datetime.fromtimestamp(seconds, tz=timezone.utc)
    return f"{dt:%Y-%m-%d %H:%M:%S}.{nanos:09d} UTC"


def fmt_path(raw):
    return raw.decode("utf-8", errors="replace")


# --------------------------------------------------------------------------
# Header and entries
# --------------------------------------------------------------------------


def parse_header(buf):
    if len(buf) < 12:
        raise IndexError_("file too short to contain a header")
    signature = buf[0:4]
    if signature != b"DIRC":
        raise IndexError_(f"bad signature {signature!r}, expected b'DIRC'")
    version, count = struct.unpack_from(">II", buf, 4)
    if version not in (2, 3, 4):
        raise IndexError_(f"unsupported index version {version}")
    return version, count


def parse_entries(buf, version, count, hash_len):
    """Parse all index entries. Returns (entries, offset_after_entries)."""
    entries = []
    off = 12
    prev_path = b""

    for _ in range(count):
        start = off

        (ctime_s, ctime_ns, mtime_s, mtime_ns, dev, ino, mode, uid, gid, size) = (
            struct.unpack_from(">10I", buf, off)
        )
        off += 40

        oid = buf[off : off + hash_len].hex()
        off += hash_len

        (flags,) = struct.unpack_from(">H", buf, off)
        off += 2

        assume_valid = bool(flags & 0x8000)
        extended = bool(flags & 0x4000)
        stage = (flags >> 12) & 0x3
        name_len = flags & 0x0FFF

        skip_worktree = intent_to_add = False
        if extended:
            if version < 3:
                raise IndexError_("extended flag set in a version 2 index")
            (ext_flags,) = struct.unpack_from(">H", buf, off)
            off += 2
            skip_worktree = bool(ext_flags & 0x4000)
            intent_to_add = bool(ext_flags & 0x2000)

        if version == 4:
            # Path is prefix-compressed against the previous entry's path.
            strip, off = read_varint(buf, off)
            suffix, off = read_cstring(buf, off)
            path = prev_path[: len(prev_path) - strip] + suffix
            prev_path = path
        else:
            if name_len == 0x0FFF:
                # Name is 4095 bytes or longer: find the real length via NUL.
                name_len = buf.index(b"\x00", off) - off
            path = buf[off : off + name_len]
            off += name_len
            # Pad with 1-8 NUL bytes so the whole entry is a multiple of 8.
            entry_len = off - start
            off = start + ((entry_len + 8) // 8) * 8

        entries.append(
            {
                "ctime": (ctime_s, ctime_ns),
                "mtime": (mtime_s, mtime_ns),
                "dev": dev,
                "ino": ino,
                "mode": mode,
                "uid": uid,
                "gid": gid,
                "size": size,
                "oid": oid,
                "stage": stage,
                "assume_valid": assume_valid,
                "skip_worktree": skip_worktree,
                "intent_to_add": intent_to_add,
                "path": path,
            }
        )

    return entries, off


# --------------------------------------------------------------------------
# Extensions
# --------------------------------------------------------------------------


def parse_tree_ext(data, hash_len):
    """
    Decode the TREE (cache tree) extension.
    Each node: path\\0 entry_count SP subtree_count \\n [oid if entry_count >= 0]
    Nodes appear top-down, depth-first.
    """
    pos = 0

    def node():
        nonlocal pos
        name, pos = read_cstring(data, pos)
        nl = data.index(b"\n", pos)
        count_s, subtrees_s = data[pos:nl].split(b" ")
        pos = nl + 1
        count, subtrees = int(count_s), int(subtrees_s)
        oid = None
        if count >= 0:
            oid = data[pos : pos + hash_len].hex()
            pos += hash_len
        children = [node() for _ in range(subtrees)]
        return {"name": name, "count": count, "oid": oid, "children": children}

    return node() if data else None


def print_tree_node(n, depth=0):
    indent = "    " * depth
    name = fmt_path(n["name"]) or "(root)"
    if n["count"] < 0:
        info = "invalidated"
    else:
        info = f"{n['count']} entries -> tree {n['oid']}"
    print(f"      {indent}{name}/  [{info}]")
    for c in n["children"]:
        print_tree_node(c, depth + 1)


def parse_reuc_ext(data, hash_len):
    """Decode REUC (resolve undo): path\\0 three octal modes (\\0 each) + oids."""
    pos = 0
    items = []
    while pos < len(data):
        path, pos = read_cstring(data, pos)
        modes = []
        for _ in range(3):
            m, pos = read_cstring(data, pos)
            modes.append(int(m, 8))
        oids = []
        for m in modes:
            if m != 0:
                oids.append(data[pos : pos + hash_len].hex())
                pos += hash_len
            else:
                oids.append(None)
        items.append((path, modes, oids))
    return items


def parse_extensions(buf, off, end, hash_len, decode=True):
    exts = []
    while off < end:
        if off + 8 > end:
            raise IndexError_("truncated extension header")
        sig = buf[off : off + 4]
        (size,) = struct.unpack_from(">I", buf, off + 4)
        data = buf[off + 8 : off + 8 + size]
        if len(data) != size:
            raise IndexError_(f"extension {sig!r} runs past end of file")
        optional = 65 <= sig[0] <= 90  # first byte 'A'..'Z'
        exts.append((sig, size, optional, data))
        off += 8 + size
    return exts


def print_extensions(exts, hash_len, decode):
    if not exts:
        print("Extensions: none")
        return
    print(f"Extensions: {len(exts)}")
    for sig, size, optional, data in exts:
        name = sig.decode("ascii", errors="replace")
        kind = "optional" if optional else "REQUIRED"
        print(f"  {name}  ({size} bytes, {kind})")
        if not decode:
            continue
        try:
            if sig == b"TREE":
                root = parse_tree_ext(data, hash_len)
                if root:
                    print_tree_node(root)
            elif sig == b"REUC":
                for path, modes, oids in parse_reuc_ext(data, hash_len):
                    print(f"      {fmt_path(path)}")
                    for stage, (m, o) in enumerate(zip(modes, oids), start=1):
                        if o:
                            print(f"        stage {stage}: {m:06o} {o}")
            elif sig == b"EOIE":
                (offset,) = struct.unpack_from(">I", data, 0)
                print(f"      entries end at byte offset {offset}")
            elif sig == b"IEOT":
                (ver,) = struct.unpack_from(">I", data, 0)
                blocks = (len(data) - 4) // 8
                print(f"      version {ver}, {blocks} block(s)")
                for i in range(blocks):
                    o, c = struct.unpack_from(">II", data, 4 + i * 8)
                    print(f"        block {i}: offset {o}, {c} entries")
            # Other extensions (UNTR, FSMN, link, sdir) are only listed.
        except (ValueError, IndexError, struct.error) as exc:
            print(f"      (could not decode: {exc})")


# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------


def describe_mode(mode):
    obj_type = (mode >> 12) & 0xF
    perms = mode & 0o777
    return f"{mode:06o} ({OBJECT_TYPES.get(obj_type, f'unknown type {obj_type:04b}')}, perms {perms:04o})"


def print_short(entries):
    for e in entries:
        print(f"{e['mode']:06o} {e['oid']} {e['stage']}\t{fmt_path(e['path'])}")


def print_full(entries):
    for i, e in enumerate(entries):
        print(f"[{i}] {fmt_path(e['path'])}")
        print(f"    mode:   {describe_mode(e['mode'])}")
        print(f"    object: {e['oid']}")
        print(f"    stage:  {e['stage']}")
        print(f"    size:   {e['size']}")
        print(f"    ctime:  {fmt_time(*e['ctime'])}")
        print(f"    mtime:  {fmt_time(*e['mtime'])}")
        print(f"    dev/ino: {e['dev']} / {e['ino']}")
        print(f"    uid/gid: {e['uid']} / {e['gid']}")
        flags = [
            name
            for name, on in (
                ("assume-valid", e["assume_valid"]),
                ("skip-worktree", e["skip_worktree"]),
                ("intent-to-add", e["intent_to_add"]),
            )
            if on
        ]
        if flags:
            print(f"    flags:  {', '.join(flags)}")
        print()


# --------------------------------------------------------------------------
# Main
# --------------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(description="Read and display a Git index file.")
    ap.add_argument(
        "path",
        nargs="?",
        default=".git/index",
        help="path to the index file (default: .git/index)",
    )
    ap.add_argument(
        "--short",
        action="store_true",
        help="one line per entry, like `git ls-files --stage`",
    )
    ap.add_argument(
        "--no-ext", action="store_true", help="do not decode extension contents"
    )
    ap.add_argument(
        "--hash",
        choices=("sha1", "sha256"),
        default="sha1",
        help="hash algorithm of the repository (default: sha1)",
    )
    args = ap.parse_args()

    hash_len = 20 if args.hash == "sha1" else 32

    try:
        with open(args.path, "rb") as f:
            buf = f.read()
    except OSError as exc:
        sys.exit(f"error: cannot read {args.path}: {exc}")

    try:
        version, count = parse_header(buf)
        entries, off = parse_entries(buf, version, count, hash_len)
        end = len(buf) - hash_len
        if off > end:
            raise IndexError_("entries run into the checksum; wrong --hash?")
        exts = parse_extensions(buf, off, end, hash_len)
    except (IndexError_, struct.error, ValueError, IndexError) as exc:
        sys.exit(f"error: malformed index: {exc}")

    if args.short:
        print_short(entries)
        return

    print(f"File:     {args.path} ({len(buf)} bytes)")
    print(f"Version:  {version}")
    print(f"Entries:  {count}")
    print()
    print_full(entries)
    print_extensions(exts, hash_len, decode=not args.no_ext)

    stored = buf[end:].hex()
    computed = hashlib.new(args.hash, buf[:end]).hexdigest()
    status = "OK" if stored == computed else f"MISMATCH (computed {computed})"
    print()
    print(f"Checksum: {stored}  [{status}]")


if __name__ == "__main__":
    main()
