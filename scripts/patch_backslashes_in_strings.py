#!/usr/bin/env python3
"""Patch backslashes in *strings* inside a binary.

Goal:
  Replace every '\\' with '/' *only* when it is inside a likely string,
  and (by default) only inside non-executable, string-bearing sections.

Supported formats:
  - ELF (32/64-bit, little/big endian)
  - PE/COFF (Windows EXE/DLL/EFI)

What counts as a "string":
  - ASCII/UTF-8-ish C strings (printable bytes, NUL-terminated)
  - UTF-16LE NUL-terminated strings (common in PE debug paths)

Safety:
  - Replacements are same-length, in-place byte edits (no relocations).
  - Limits scanning to selected sections/regions (format-aware).
  - Requires NUL termination and a high printable ratio.

Usage examples:
  Dry-run (recommended first):
    python scripts/patch_backslashes_in_strings.py path/to/binary --dry-run

  Write a new output file:
    python scripts/patch_backslashes_in_strings.py in.bin -o out.bin

  In-place (makes a .bak copy unless --no-backup):
    python scripts/patch_backslashes_in_strings.py in.bin --in-place

Notes:
  This is a best-effort heuristic patcher. Use --dry-run and inspect the
  report before applying in-place.
"""

from __future__ import annotations

import argparse
import dataclasses
import os
import struct
import sys
from typing import Iterable, List, Optional, Sequence, Tuple


PRINTABLE_ASCII = set(range(0x20, 0x7F))
# Add common whitespace allowed inside paths/strings
ALLOWED_ASCII_EXTRA = {0x09, 0x0A, 0x0D}


@dataclasses.dataclass(frozen=True)
class Region:
    name: str
    offset: int
    size: int


@dataclasses.dataclass
class Patch:
    offset: int
    old: bytes
    new: bytes
    kind: str
    region: str


def read_u16(data: bytes, offset: int, endian: str) -> int:
    return struct.unpack_from(endian + "H", data, offset)[0]


def read_u32(data: bytes, offset: int, endian: str) -> int:
    return struct.unpack_from(endian + "I", data, offset)[0]


def read_u64(data: bytes, offset: int, endian: str) -> int:
    return struct.unpack_from(endian + "Q", data, offset)[0]


def is_printable_byte(b: int) -> bool:
    return (b in PRINTABLE_ASCII) or (b in ALLOWED_ASCII_EXTRA)


def looks_like_ascii_c_string(buf: bytes, start: int, end: int, min_len: int, printable_ratio: float) -> Optional[int]:
    """If buf[start:] begins with a likely ASCII C-string, return its length (excluding NUL)."""
    if start >= end:
        return None

    i = start
    printable = 0
    total = 0

    while i < end:
        b = buf[i]
        if b == 0:
            break
        total += 1
        if is_printable_byte(b):
            printable += 1
        else:
            # Reject early if non-printable appears.
            return None
        i += 1

    if i >= end:
        return None  # no terminator in region

    length = i - start
    if length < min_len:
        return None

    if total == 0:
        return None

    if (printable / total) < printable_ratio:
        return None

    return length


def looks_like_utf16le_c_string(buf: bytes, start: int, end: int, min_len: int, printable_ratio: float) -> Optional[int]:
    """If buf[start:] begins with a likely UTF-16LE NUL-terminated string, return byte length (excluding 2-byte NUL)."""
    if start + 2 > end:
        return None
    if start % 2 != 0:
        return None

    i = start
    printable = 0
    total = 0

    while i + 1 < end:
        lo = buf[i]
        hi = buf[i + 1]
        if lo == 0 and hi == 0:
            break
        total += 1
        # Common case: ASCII subset in UTF-16LE -> high byte 0
        if hi != 0:
            return None
        if is_printable_byte(lo):
            printable += 1
        else:
            return None
        i += 2

    if i + 1 >= end:
        return None

    char_len = (i - start) // 2
    if char_len < min_len:
        return None

    if total == 0:
        return None

    if (printable / total) < printable_ratio:
        return None

    return i - start


def scan_and_patch_region(
    buf: bytearray,
    region: Region,
    min_len: int,
    printable_ratio: float,
    patch_utf16: bool,
    patch_ascii: bool,
) -> List[Patch]:
    patches: List[Patch] = []

    start = region.offset
    end = region.offset + region.size

    if start < 0 or end > len(buf) or start >= end:
        return patches

    i = start
    while i < end:
        if patch_ascii:
            ascii_len = looks_like_ascii_c_string(buf, i, end, min_len=min_len, printable_ratio=printable_ratio)
            if ascii_len is not None:
                raw = bytes(buf[i : i + ascii_len])
                if b"\\" in raw:
                    new = raw.replace(b"\\", b"/")
                    if new != raw:
                        patches.append(Patch(offset=i, old=raw, new=new, kind="ascii", region=region.name))
                        buf[i : i + ascii_len] = new
                i = i + ascii_len + 1  # skip NUL terminator
                continue

        if patch_utf16 and (i % 2 == 0):
            utf16_len = looks_like_utf16le_c_string(buf, i, end, min_len=min_len, printable_ratio=printable_ratio)
            if utf16_len is not None:
                raw = bytes(buf[i : i + utf16_len])
                # backslash in UTF-16LE ASCII subset is b'\\\x00'
                if b"\\\x00" in raw:
                    new = raw.replace(b"\\\x00", b"/\x00")
                    if new != raw:
                        patches.append(Patch(offset=i, old=raw, new=new, kind="utf16le", region=region.name))
                        buf[i : i + utf16_len] = new
                i = i + utf16_len + 2  # skip 2-byte terminator
                continue

        i += 1

    return patches


# ---------------- ELF parsing ----------------


def elf_regions(data: bytes) -> List[Region]:
    # Minimal ELF section header parsing.
    # https://refspecs.linuxfoundation.org/elf/

    if len(data) < 0x40:
        raise ValueError("File too small for ELF")

    if data[0:4] != b"\x7fELF":
        raise ValueError("Not an ELF file")

    ei_class = data[4]  # 1=32, 2=64
    ei_data = data[5]   # 1=little, 2=big

    if ei_class not in (1, 2):
        raise ValueError(f"Unknown ELF class: {ei_class}")
    if ei_data not in (1, 2):
        raise ValueError(f"Unknown ELF endianness: {ei_data}")

    endian = "<" if ei_data == 1 else ">"

    if ei_class == 2:
        # ELF64_Ehdr
        e_shoff = read_u64(data, 0x28, endian)
        e_shentsize = read_u16(data, 0x3A, endian)
        e_shnum = read_u16(data, 0x3C, endian)
        e_shstrndx = read_u16(data, 0x3E, endian)
        sh_fmt = endian + "IIQQQQIIQQ"  # Elf64_Shdr
        sh_size = struct.calcsize(sh_fmt)
    else:
        # ELF32_Ehdr
        e_shoff = read_u32(data, 0x20, endian)
        e_shentsize = read_u16(data, 0x2E, endian)
        e_shnum = read_u16(data, 0x30, endian)
        e_shstrndx = read_u16(data, 0x32, endian)
        sh_fmt = endian + "IIIIIIIIII"  # Elf32_Shdr
        sh_size = struct.calcsize(sh_fmt)

    if e_shentsize == 0 or e_shnum == 0:
        return []

    # Some linkers set e_shentsize == struct size; trust file value but validate.
    if e_shentsize < sh_size:
        raise ValueError("ELF section header entry size too small")

    sh_table_end = e_shoff + (e_shentsize * e_shnum)
    if sh_table_end > len(data):
        raise ValueError("ELF section header table out of range")

    # Read shstrtab to resolve section names
    def read_shdr(idx: int) -> Tuple[int, int, int, int]:
        base = e_shoff + idx * e_shentsize
        # unpack only the fields we need: sh_name, sh_type, sh_flags, sh_offset, sh_size
        if ei_class == 2:
            sh_name, sh_type, sh_flags, _sh_addr, sh_offset, sh_sz, _sh_link, _sh_info, _sh_addralign, _sh_entsize = struct.unpack_from(sh_fmt, data, base)
        else:
            sh_name, sh_type, sh_flags, _sh_addr, sh_offset, sh_sz, _sh_link, _sh_info, _sh_addralign, _sh_entsize = struct.unpack_from(sh_fmt, data, base)
        return sh_name, sh_type, sh_flags, sh_offset, sh_sz

    if e_shstrndx == 0 or e_shstrndx >= e_shnum:
        # no section names; still choose conservative regions by flags/type
        shstr = b""
        shstr_off = 0
    else:
        sh_name, sh_type, sh_flags, sh_offset, sh_sz = read_shdr(e_shstrndx)
        shstr_off = sh_offset
        shstr = data[sh_offset : sh_offset + sh_sz]

    def get_name(name_off: int) -> str:
        if not shstr or name_off >= len(shstr):
            return f"section_{name_off}"
        end = shstr.find(b"\x00", name_off)
        if end == -1:
            end = len(shstr)
        try:
            return shstr[name_off:end].decode("utf-8", errors="replace")
        except Exception:
            return f"section_{name_off}"

    # Filter regions:
    # - Prefer debug/string tables: .debug_*, .strtab, .symtab (names only)
    # - Also allow non-exec PROGBITS that are likely rodata: .rodata
    # - Never patch executable sections (SHF_EXECINSTR)
    SHF_EXECINSTR = 0x4

    regions: List[Region] = []
    for idx in range(e_shnum):
        sh_name, sh_type, sh_flags, sh_offset, sh_sz = read_shdr(idx)
        name = get_name(sh_name)

        if sh_sz == 0:
            continue
        if sh_offset == 0 or sh_offset + sh_sz > len(data):
            continue
        if (sh_flags & SHF_EXECINSTR) != 0:
            continue

        is_debug = name.startswith(".debug")
        is_str = name in (".strtab", ".debug_str", ".debug_line_str")
        is_rodata = name in (".rodata", ".rdata")

        if is_debug or is_str or is_rodata:
            regions.append(Region(name=name, offset=sh_offset, size=sh_sz))

    return regions


# ---------------- PE parsing ----------------


def pe_regions(data: bytes) -> List[Region]:
    # Minimal PE/COFF parsing (PE32/PE32+).
    if len(data) < 0x100:
        raise ValueError("File too small for PE")
    if data[0:2] != b"MZ":
        raise ValueError("Not a PE file")

    e_lfanew = read_u32(data, 0x3C, "<")
    if e_lfanew + 4 > len(data):
        raise ValueError("PE header out of range")
    if data[e_lfanew : e_lfanew + 4] != b"PE\x00\x00":
        raise ValueError("Missing PE signature")

    coff_off = e_lfanew + 4
    if coff_off + 20 > len(data):
        raise ValueError("COFF header out of range")

    number_of_sections = read_u16(data, coff_off + 2, "<")
    size_of_optional_header = read_u16(data, coff_off + 16, "<")

    opt_off = coff_off + 20
    sect_off = opt_off + size_of_optional_header
    if sect_off + (40 * number_of_sections) > len(data):
        raise ValueError("Section headers out of range")

    regions: List[Region] = []

    # Heuristic: patch only sections that commonly contain strings/paths.
    # Avoid .text (code). Prefer .rdata/.data and debug-related sections.
    for i in range(number_of_sections):
        base = sect_off + i * 40
        name_raw = data[base : base + 8]
        name = name_raw.split(b"\x00", 1)[0].decode("ascii", errors="replace")

        virtual_size = read_u32(data, base + 8, "<")
        raw_size = read_u32(data, base + 16, "<")
        raw_ptr = read_u32(data, base + 20, "<")

        size = raw_size if raw_size != 0 else virtual_size
        if size == 0:
            continue
        if raw_ptr == 0 or raw_ptr + size > len(data):
            continue

        if name.lower().startswith(".text"):
            continue

        allow = name.lower() in (".rdata", ".data") or name.lower().startswith(".debug")
        if allow:
            regions.append(Region(name=name, offset=raw_ptr, size=size))

    return regions


def detect_regions(data: bytes) -> Tuple[str, List[Region]]:
    if data.startswith(b"\x7fELF"):
        return "ELF", elf_regions(data)
    if data.startswith(b"MZ"):
        return "PE", pe_regions(data)
    raise ValueError("Unsupported file format (expected ELF or PE)")


def format_preview(old: bytes, new: bytes, kind: str) -> str:
    if kind == "utf16le":
        try:
            old_s = old.decode("utf-16le", errors="replace")
            new_s = new.decode("utf-16le", errors="replace")
            return f"{old_s}  ->  {new_s}"
        except Exception:
            return f"{old!r} -> {new!r}"
    else:
        try:
            old_s = old.decode("utf-8", errors="replace")
            new_s = new.decode("utf-8", errors="replace")
            return f"{old_s}  ->  {new_s}"
        except Exception:
            return f"{old!r} -> {new!r}"


def main(argv: Sequence[str]) -> int:
    ap = argparse.ArgumentParser(description="Replace backslashes with slashes inside strings in ELF/PE binaries (heuristic + section-limited).")
    ap.add_argument("input", help="Input binary path")
    ap.add_argument("-o", "--output", help="Output path (default: required unless --in-place)")
    ap.add_argument("--in-place", action="store_true", help="Modify the input file in-place")
    ap.add_argument("--no-backup", action="store_true", help="When using --in-place, do not create a .bak backup")
    ap.add_argument("--dry-run", action="store_true", help="Do not write output; just report what would change")
    ap.add_argument("--min-len", type=int, default=6, help="Minimum string length to consider (default: 6)")
    ap.add_argument("--printable-ratio", type=float, default=1.0, help="Printable ratio threshold (0-1, default: 1.0)")
    ap.add_argument("--no-utf16", action="store_true", help="Disable UTF-16LE scanning")
    ap.add_argument("--no-ascii", action="store_true", help="Disable ASCII scanning")
    ap.add_argument("--max-report", type=int, default=50, help="Max patches to print (default: 50)")

    args = ap.parse_args(argv)

    in_path = args.input
    if not os.path.isfile(in_path):
        print(f"error: input file does not exist: {in_path}", file=sys.stderr)
        return 2

    if args.in_place:
        out_path = in_path
        if args.output:
            print("error: do not use --output with --in-place", file=sys.stderr)
            return 2
    else:
        if not args.output and not args.dry_run:
            print("error: specify --output or use --in-place (or --dry-run)", file=sys.stderr)
            return 2
        out_path = args.output

    data = bytearray(open(in_path, "rb").read())

    try:
        fmt, regions = detect_regions(bytes(data))
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    if not regions:
        print(f"warning: detected {fmt} but found no safe regions to scan.")

    all_patches: List[Patch] = []
    for region in regions:
        patches = scan_and_patch_region(
            data,
            region,
            min_len=args.min_len,
            printable_ratio=args.printable_ratio,
            patch_utf16=not args.no_utf16,
            patch_ascii=not args.no_ascii,
        )
        all_patches.extend(patches)

    print(f"format: {fmt}")
    print(f"regions scanned: {len(regions)}")
    print(f"strings patched: {len(all_patches)}")

    if all_patches:
        print("\nPatch preview (truncated):")
        for p in all_patches[: args.max_report]:
            preview = format_preview(p.old, p.new, p.kind)
            print(f"- {p.kind} @{p.offset:#x} ({p.region}): {preview}")
        if len(all_patches) > args.max_report:
            print(f"  ... {len(all_patches) - args.max_report} more")

    if args.dry_run:
        return 0

    if args.in_place and not args.no_backup:
        bak = in_path + ".bak"
        if not os.path.exists(bak):
            with open(bak, "wb") as f:
                f.write(bytes(open(in_path, "rb").read()))
            print(f"backup written: {bak}")
        else:
            print(f"backup exists, not overwriting: {bak}")

    if out_path is None:
        print("error: no output path", file=sys.stderr)
        return 2

    with open(out_path, "wb") as f:
        f.write(data)

    print(f"wrote: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
